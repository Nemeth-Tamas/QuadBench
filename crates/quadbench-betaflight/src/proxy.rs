use std::{
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use tracing::{debug, info, warn};
use tungstenite::{
    Error as WebSocketError, Message, accept_hdr,
    handshake::server::{Request, Response},
};

#[derive(Debug, Clone, Copy)]
pub struct ConfiguratorProxyConfig {
    pub listen_host: IpAddr,
    pub listen_port: u16,
    pub target_host: IpAddr,
    pub target_port: u16,
}

impl Default for ConfiguratorProxyConfig {
    fn default() -> Self {
        let target_host = std::env::var("QUADBENCH_SITL_HOST")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

        Self {
            listen_host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            listen_port: 6_761,
            target_host,
            target_port: 5_761,
        }
    }
}

impl ConfiguratorProxyConfig {
    pub fn listen_address(self) -> SocketAddr {
        SocketAddr::new(self.listen_host, self.listen_port)
    }

    pub fn target_address(self) -> SocketAddr {
        SocketAddr::new(self.target_host, self.target_port)
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguratorProxySnapshot {
    pub config: ConfiguratorProxyConfig,
    pub client_connected: bool,
    pub sitl_connected: bool,
    pub sessions: u64,
    pub handshake_attempts: u64,
    pub successful_handshakes: u64,
    pub last_requested_protocol: Option<String>,
    pub negotiated_protocol: Option<String>,
    pub bytes_from_configurator: u64,
    pub bytes_from_sitl: u64,
    pub last_error: Option<String>,
}

impl ConfiguratorProxySnapshot {
    pub fn connected(&self) -> bool {
        self.client_connected && self.sitl_connected
    }

    pub fn websocket_url(&self) -> String {
        format!("ws://{}", self.config.listen_address(),)
    }
}

struct ProxyTelemetry {
    client_connected: bool,
    sitl_connected: bool,
    sessions: u64,
    handshake_attempts: u64,
    successful_handshakes: u64,
    last_requested_protocol: Option<String>,
    negotiated_protocol: Option<String>,
    bytes_from_configurator: u64,
    bytes_from_sitl: u64,
    last_error: Option<String>,
}

pub struct ConfiguratorProxy {
    config: ConfiguratorProxyConfig,
    telemetry: Arc<Mutex<ProxyTelemetry>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl ConfiguratorProxy {
    pub fn spawn(config: ConfiguratorProxyConfig) -> io::Result<Self> {
        let listener = TcpListener::bind(config.listen_address())?;

        listener.set_nonblocking(true)?;

        let telemetry = Arc::new(Mutex::new(ProxyTelemetry {
            client_connected: false,
            sitl_connected: false,
            sessions: 0,
            handshake_attempts: 0,
            successful_handshakes: 0,
            last_requested_protocol: None,
            negotiated_protocol: None,
            bytes_from_configurator: 0,
            bytes_from_sitl: 0,
            last_error: None,
        }));

        let stop = Arc::new(AtomicBool::new(false));

        let worker_telemetry = Arc::clone(&telemetry);

        let worker_stop = Arc::clone(&stop);

        let worker = thread::Builder::new()
            .name("quadbench-configurator-proxy".to_owned())
            .spawn(move || {
                run_proxy(config, listener, worker_telemetry, worker_stop);
            })?;

        info!(
            listen = %config.listen_address(),
            target = %config.target_address(),
            "Configurator WebSocket proxy ready"
        );

        Ok(Self {
            config,
            telemetry,
            stop,
            worker: Some(worker),
        })
    }

    pub fn snapshot(&self) -> ConfiguratorProxySnapshot {
        let telemetry = self
            .telemetry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        ConfiguratorProxySnapshot {
            config: self.config,
            client_connected: telemetry.client_connected,
            sitl_connected: telemetry.sitl_connected,
            sessions: telemetry.sessions,
            handshake_attempts: telemetry.handshake_attempts,
            successful_handshakes: telemetry.successful_handshakes,
            last_requested_protocol: telemetry.last_requested_protocol.clone(),
            negotiated_protocol: telemetry.negotiated_protocol.clone(),
            bytes_from_configurator: telemetry.bytes_from_configurator,
            bytes_from_sitl: telemetry.bytes_from_sitl,
            last_error: telemetry.last_error.clone(),
        }
    }
}

impl Drop for ConfiguratorProxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);

        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                warn!("Configurator proxy worker panicked during shutdown");
            }
        }
    }
}

fn run_proxy(
    config: ConfiguratorProxyConfig,
    listener: TcpListener,
    telemetry: Arc<Mutex<ProxyTelemetry>>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, peer)) => {
                info!(
                    %peer,
                    "Configurator WebSocket client connected"
                );

                if let Err(error) = handle_client(config, stream, &telemetry, &stop) {
                    record_error(&telemetry, error);
                }

                set_connection_state(&telemetry, false, false);

                info!(
                    %peer,
                    "Configurator WebSocket client disconnected"
                );
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                record_error(&telemetry, format!("proxy accept failed: {error}"));

                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn handle_client(
    config: ConfiguratorProxyConfig,
    client_stream: TcpStream,
    telemetry: &Arc<Mutex<ProxyTelemetry>>,
    stop: &Arc<AtomicBool>,
) -> Result<(), String> {
    client_stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("failed to configure WebSocket handshake timeout: {error}"))?;

    client_stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("failed to configure WebSocket write timeout: {error}"))?;

    {
        let mut telemetry = telemetry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        telemetry.handshake_attempts += 1;
    }

    let handshake_telemetry = Arc::clone(telemetry);

    let callback = move |request: &Request, mut response: Response| {
        let requested_protocol = request
            .headers()
            .get("Sec-WebSocket-Protocol")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);

        let binary_requested = requests_binary_subprotocol(requested_protocol.as_deref());

        {
            let mut telemetry = handshake_telemetry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            telemetry.last_requested_protocol = requested_protocol;

            telemetry.negotiated_protocol = if binary_requested {
                Some("binary".to_owned())
            } else {
                None
            };
        }

        if binary_requested {
            response.headers_mut().append(
                "Sec-WebSocket-Protocol",
                "binary"
                    .parse()
                    .expect("binary is a valid WebSocket subprotocol header"),
            );
        }

        Ok(response)
    };

    let mut websocket = accept_hdr(client_stream, callback)
        .map_err(|error| format!("WebSocket handshake failed: {error}"))?;

    {
        let mut telemetry = telemetry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        telemetry.successful_handshakes += 1;
    }

    websocket
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(5)))
        .map_err(|error| format!("failed to configure WebSocket read timeout: {error}"))?;

    websocket
        .get_mut()
        .set_write_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| format!("failed to configure WebSocket write timeout: {error}"))?;

    set_connection_state(telemetry, true, false);

    let target_address = config.target_address();

    let mut sitl_stream = TcpStream::connect_timeout(&target_address, Duration::from_secs(2))
        .map_err(|error| {
            format!("failed to connect to Betaflight SITL at {target_address}: {error}")
        })?;

    sitl_stream
        .set_read_timeout(Some(Duration::from_millis(5)))
        .map_err(|error| format!("failed to configure SITL read timeout: {error}"))?;

    sitl_stream
        .set_write_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| format!("failed to configure SITL write timeout: {error}"))?;

    {
        let mut telemetry = telemetry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        telemetry.client_connected = true;
        telemetry.sitl_connected = true;
        telemetry.sessions += 1;
        telemetry.last_error = None;
    }

    info!(
        %target_address,
        "Configurator proxy connected to Betaflight SITL"
    );

    let mut sitl_buffer = [0_u8; 16_384];

    while !stop.load(Ordering::Acquire) {
        match websocket.read() {
            Ok(message) if message.is_binary() || message.is_text() => {
                let data = message.into_data();

                if !data.is_empty() {
                    sitl_stream
                        .write_all(&data)
                        .map_err(|error| format!("Configurator -> SITL write failed: {error}"))?;

                    let mut telemetry = telemetry
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    telemetry.bytes_from_configurator += data.len() as u64;
                }
            }
            Ok(message) if message.is_close() => {
                break;
            }
            Ok(_) => {}
            Err(WebSocketError::ConnectionClosed | WebSocketError::AlreadyClosed) => {
                break;
            }
            Err(WebSocketError::Io(error)) if is_retryable_io(&error) => {}
            Err(error) => {
                return Err(format!("WebSocket read failed: {error}"));
            }
        }

        match sitl_stream.read(&mut sitl_buffer) {
            Ok(0) => {
                debug!("Betaflight SITL closed MSP connection");

                break;
            }
            Ok(length) => {
                websocket
                    .send(Message::from(&sitl_buffer[..length]))
                    .map_err(|error| {
                        format!("SITL -> Configurator WebSocket write failed: {error}")
                    })?;

                let mut telemetry = telemetry
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                telemetry.bytes_from_sitl += length as u64;
            }
            Err(error) if is_retryable_io(&error) => {}
            Err(error) => {
                return Err(format!("SITL -> Configurator read failed: {error}"));
            }
        }

        match websocket.flush() {
            Ok(()) => {}
            Err(WebSocketError::ConnectionClosed | WebSocketError::AlreadyClosed) => {
                break;
            }
            Err(WebSocketError::Io(error)) if is_retryable_io(&error) => {}
            Err(error) => {
                return Err(format!("WebSocket flush failed: {error}"));
            }
        }

        thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn is_retryable_io(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

fn set_connection_state(
    telemetry: &Arc<Mutex<ProxyTelemetry>>,
    client_connected: bool,
    sitl_connected: bool,
) {
    let mut telemetry = telemetry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    telemetry.client_connected = client_connected;

    telemetry.sitl_connected = sitl_connected;
}

fn record_error(telemetry: &Arc<Mutex<ProxyTelemetry>>, message: String) {
    warn!(
        error = %message,
        "Configurator proxy error"
    );

    let mut telemetry = telemetry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    telemetry.last_error = Some(message);
}

fn requests_binary_subprotocol(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value
            .split(',')
            .any(|protocol| protocol.trim().eq_ignore_ascii_case("binary"))
    })
}

#[cfg(test)]
mod tests {
    use super::requests_binary_subprotocol;

    #[test]
    fn detects_binary_subprotocol() {
        assert!(requests_binary_subprotocol(Some("binary")));
        assert!(requests_binary_subprotocol(Some("foo, binary")));
        assert!(requests_binary_subprotocol(Some("Binary")));
    }

    #[test]
    fn rejects_missing_binary_subprotocol() {
        assert!(!requests_binary_subprotocol(None));
        assert!(!requests_binary_subprotocol(Some("")));
        assert!(!requests_binary_subprotocol(Some("wsSerial")));
    }
}
