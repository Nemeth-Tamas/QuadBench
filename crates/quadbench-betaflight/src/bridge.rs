use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use tracing::{debug, info, warn};

use crate::protocol::{
    FdmPacket, MOTOR_COUNT, RC_CHANNEL_COUNT, decode_motor_packet, encode_rc_packet,
};

#[derive(Debug, Clone, Copy)]
pub struct SitlConfig {
    pub target_host: IpAddr,
    pub motor_port: u16,
    pub fdm_port: u16,
    pub rc_port: u16,
    pub fdm_rate_hz: u32,
    pub rc_rate_hz: u32,
}

impl Default for SitlConfig {
    fn default() -> Self {
        let target_host = std::env::var("QUADBENCH_SITL_HOST")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

        Self {
            target_host,
            motor_port: 9_002,
            fdm_port: 9_003,
            rc_port: 9_004,
            fdm_rate_hz: 250,
            rc_rate_hz: 250,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SitlSnapshot {
    pub config: SitlConfig,
    pub enabled: bool,
    pub receiver_streaming: bool,
    pub tx_fdm_packets: u64,
    pub tx_rc_packets: u64,
    pub rx_motor_packets: u64,
    pub motor_commands: [f32; MOTOR_COUNT],
    pub last_rc_packet_age: Option<Duration>,
    pub last_motor_packet_age: Option<Duration>,
    pub last_error: Option<String>,
}

impl SitlSnapshot {
    pub fn connected(&self) -> bool {
        self.enabled
            && self
                .last_motor_packet_age
                .is_some_and(|age| age <= Duration::from_secs(1))
    }
}

struct BridgeControl {
    enabled: bool,
    receiver_enabled: bool,
    channels: [u16; RC_CHANNEL_COUNT],
}

struct BridgeTelemetry {
    tx_fdm_packets: u64,
    tx_rc_packets: u64,
    rx_motor_packets: u64,
    motor_commands: [f32; MOTOR_COUNT],
    last_rc_packet: Option<Instant>,
    last_motor_packet: Option<Instant>,
    last_error: Option<String>,
}

pub struct SitlBridge {
    config: SitlConfig,
    control: Arc<Mutex<BridgeControl>>,
    telemetry: Arc<Mutex<BridgeTelemetry>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl SitlBridge {
    pub fn spawn(config: SitlConfig) -> io::Result<Self> {
        let motor_bind_address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.motor_port);

        let motor_socket = UdpSocket::bind(motor_bind_address)?;

        motor_socket.set_nonblocking(true)?;

        let tx_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))?;

        let control = Arc::new(Mutex::new(BridgeControl {
            enabled: false,
            receiver_enabled: false,
            channels: [1_500; RC_CHANNEL_COUNT],
        }));

        let telemetry = Arc::new(Mutex::new(BridgeTelemetry {
            tx_fdm_packets: 0,
            tx_rc_packets: 0,
            rx_motor_packets: 0,
            motor_commands: [0.0; MOTOR_COUNT],
            last_rc_packet: None,
            last_motor_packet: None,
            last_error: None,
        }));

        let stop = Arc::new(AtomicBool::new(false));

        let worker_control = Arc::clone(&control);

        let worker_telemetry = Arc::clone(&telemetry);

        let worker_stop = Arc::clone(&stop);

        let worker = thread::Builder::new()
            .name("quadbench-sitl".to_owned())
            .spawn(move || {
                run_worker(
                    config,
                    motor_socket,
                    tx_socket,
                    worker_control,
                    worker_telemetry,
                    worker_stop,
                );
            })?;

        info!(
            motor_port = config.motor_port,
            fdm_port = config.fdm_port,
            rc_port = config.rc_port,
            target = %config.target_host,
            "Betaflight SITL bridge ready"
        );

        Ok(Self {
            config,
            control,
            telemetry,
            stop,
            worker: Some(worker),
        })
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        control.enabled = enabled;
    }

    pub fn set_receiver_enabled(&self, enabled: bool) {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        control.receiver_enabled = enabled;
    }

    pub fn set_channels(&self, channels: [u16; RC_CHANNEL_COUNT]) {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        control.channels = channels;
    }

    pub fn snapshot(&self) -> SitlSnapshot {
        let control = self
            .control
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let telemetry = self
            .telemetry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        SitlSnapshot {
            config: self.config,
            enabled: control.enabled,
            receiver_streaming: control.enabled && control.receiver_enabled,
            tx_fdm_packets: telemetry.tx_fdm_packets,
            tx_rc_packets: telemetry.tx_rc_packets,
            rx_motor_packets: telemetry.rx_motor_packets,
            motor_commands: telemetry.motor_commands,
            last_rc_packet_age: telemetry.last_rc_packet.map(|instant| instant.elapsed()),
            last_motor_packet_age: telemetry.last_motor_packet.map(|instant| instant.elapsed()),
            last_error: telemetry.last_error.clone(),
        }
    }
}

impl Drop for SitlBridge {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);

        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                warn!("Betaflight SITL worker panicked during shutdown");
            }
        }
    }
}

fn run_worker(
    config: SitlConfig,
    motor_socket: UdpSocket,
    tx_socket: UdpSocket,
    control: Arc<Mutex<BridgeControl>>,
    telemetry: Arc<Mutex<BridgeTelemetry>>,
    stop: Arc<AtomicBool>,
) {
    let rc_target = SocketAddr::new(config.target_host, config.rc_port);

    let fdm_target = SocketAddr::new(config.target_host, config.fdm_port);

    let rc_interval = Duration::from_secs_f64(1.0 / f64::from(config.rc_rate_hz.max(1)));

    let fdm_interval = Duration::from_secs_f64(1.0 / f64::from(config.fdm_rate_hz.max(1)));

    let started = Instant::now();

    let mut next_rc = Instant::now();

    let mut next_fdm = Instant::now();

    let mut motor_buffer = [0_u8; 64];

    while !stop.load(Ordering::Acquire) {
        let now = Instant::now();

        let (enabled, receiver_enabled, channels) = {
            let control = control
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            (control.enabled, control.receiver_enabled, control.channels)
        };

        if enabled && now >= next_fdm {
            let timestamp = started.elapsed().as_secs_f64();

            let packet = FdmPacket::stationary(timestamp).encode();

            match tx_socket.send_to(&packet, fdm_target) {
                Ok(_) => {
                    let mut telemetry = telemetry
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    telemetry.tx_fdm_packets += 1;
                }
                Err(error) => {
                    record_error(&telemetry, format!("FDM send failed: {error}"));
                }
            }

            next_fdm = now + fdm_interval;
        }

        if enabled && receiver_enabled && now >= next_rc {
            let timestamp = started.elapsed().as_secs_f64();

            let packet = encode_rc_packet(timestamp, &channels);

            match tx_socket.send_to(&packet, rc_target) {
                Ok(_) => {
                    let mut telemetry = telemetry
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    telemetry.tx_rc_packets += 1;
                    telemetry.last_rc_packet = Some(Instant::now());
                }
                Err(error) => {
                    record_error(&telemetry, format!("RC send failed: {error}"));
                }
            }

            next_rc = now + rc_interval;
        }

        loop {
            match motor_socket.recv_from(&mut motor_buffer) {
                Ok((length, source)) => {
                    let Some(motors) = decode_motor_packet(&motor_buffer[..length]) else {
                        debug!(
                            length,
                            %source,
                            "ignored invalid SITL motor packet"
                        );

                        continue;
                    };

                    let mut telemetry = telemetry
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    telemetry.motor_commands = motors;

                    telemetry.rx_motor_packets += 1;

                    telemetry.last_motor_packet = Some(Instant::now());

                    telemetry.last_error = None;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(error) => {
                    record_error(&telemetry, format!("Motor receive failed: {error}"));

                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(1));
    }
}

fn record_error(telemetry: &Arc<Mutex<BridgeTelemetry>>, message: String) {
    warn!(
        error = %message,
        "Betaflight SITL bridge error"
    );

    let mut telemetry = telemetry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    telemetry.last_error = Some(message);
}
