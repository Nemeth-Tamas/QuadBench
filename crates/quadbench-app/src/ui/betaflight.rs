use eframe::egui;
use quadbench_betaflight::{ConfiguratorProxySnapshot, SitlSnapshot};
use quadbench_core::state::QuadState;

pub fn show(
    ui: &mut egui::Ui,
    state: &QuadState,
    snapshot: Option<&SitlSnapshot>,
    bridge_error: Option<&str>,
    configurator_proxy: Option<&ConfiguratorProxySnapshot>,
    configurator_proxy_error: Option<&str>,
) {
    ui.heading("Betaflight SITL");
    ui.add_space(8.0);

    show_configurator_proxy(ui, configurator_proxy, configurator_proxy_error);

    ui.add_space(16.0);

    if let Some(error) = bridge_error {
        ui.group(|ui| {
            ui.strong("SITL UDP bridge initialization failed");

            ui.label(error);
        });

        return;
    }

    let Some(snapshot) = snapshot else {
        ui.label("SITL bridge has no runtime state.");

        return;
    };

    show_udp_connection(ui, state, snapshot);

    ui.add_space(12.0);

    show_streams(ui, snapshot);

    ui.add_space(12.0);

    show_motor_output(ui, snapshot);

    ui.add_space(14.0);

    ui.group(|ui| {
        ui.strong("Current simulator plant");

        ui.label(
            "Betaflight motor output now drives the \
             fixed-rate QuadBench angular plant, and \
             resulting gyro / attitude state is streamed \
             directly back into SITL.",
        );

        ui.label(
            "Start Simulation enables FDM streaming. \
             RC streaming only runs while the Pocket \
             is connected.",
        );
    });
}

fn show_configurator_proxy(
    ui: &mut egui::Ui,
    snapshot: Option<&ConfiguratorProxySnapshot>,
    proxy_error: Option<&str>,
) {
    ui.heading("Betaflight Configurator");

    ui.add_space(6.0);

    if let Some(error) = proxy_error {
        ui.group(|ui| {
            ui.strong("WebSocket proxy failed to start");

            ui.label(error);

            ui.label(
                "Check whether port 6761 is already \
                 occupied by websockify or another \
                 QuadBench instance.",
            );
        });

        return;
    }

    let Some(snapshot) = snapshot else {
        ui.label("Configurator proxy has no runtime state.");

        return;
    };

    ui.group(|ui| {
        ui.strong("Built-in WebSocket bridge");

        ui.label(format!("Configurator URL: {}", snapshot.websocket_url(),));

        ui.label(format!(
            "Forward target: {}",
            snapshot.config.target_address(),
        ));

        ui.label(format!(
            "Browser: {}",
            if snapshot.client_connected {
                "Connected"
            } else {
                "Waiting"
            },
        ));

        ui.label(format!(
            "SITL UART1: {}",
            if snapshot.sitl_connected {
                "Connected"
            } else {
                "Waiting"
            },
        ));

        ui.label(format!("Sessions: {}", snapshot.sessions,));

        ui.label(format!(
            "WebSocket handshakes: {} / {} successful",
            snapshot.successful_handshakes, snapshot.handshake_attempts,
        ));

        ui.label(format!(
            "Requested protocol: {}",
            snapshot
                .last_requested_protocol
                .as_deref()
                .unwrap_or("none"),
        ));

        ui.label(format!(
            "Negotiated protocol: {}",
            snapshot.negotiated_protocol.as_deref().unwrap_or("none"),
        ));

        ui.label(format!(
            "Configurator -> SITL: {} bytes",
            snapshot.bytes_from_configurator,
        ));

        ui.label(format!(
            "SITL -> Configurator: {} bytes",
            snapshot.bytes_from_sitl,
        ));

        ui.add_space(6.0);

        ui.hyperlink_to(
            "Open Betaflight Configurator",
            "https://app.betaflight.com/",
        );

        ui.label(
            "Enable manual connection mode and use \
             ws://127.0.0.1:6761 as the port.",
        );
    });

    if let Some(error) = snapshot.last_error.as_deref() {
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.strong("Last Configurator proxy error");

            ui.label(error);
        });
    }
}

fn show_udp_connection(ui: &mut egui::Ui, state: &QuadState, snapshot: &SitlSnapshot) {
    ui.heading("SITL UDP bridge");
    ui.add_space(6.0);

    ui.group(|ui| {
        ui.strong("Connection");

        ui.label(format!("State: {}", state.system.betaflight_link.label(),));

        ui.label(format!("Target host: {}", snapshot.config.target_host,));

        ui.label(format!("Motor output: UDP {}", snapshot.config.motor_port,));

        ui.label(format!("FDM input: UDP {}", snapshot.config.fdm_port,));

        ui.label(format!("RC input: UDP {}", snapshot.config.rc_port,));
    });

    if let Some(error) = snapshot.last_error.as_deref() {
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.strong("Last UDP transport error");

            ui.label(error);
        });
    }
}

fn show_streams(ui: &mut egui::Ui, snapshot: &SitlSnapshot) {
    ui.heading("Streams");

    egui::Grid::new("betaflight_stream_grid")
        .num_columns(3)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Stream");
            ui.strong("State");
            ui.strong("Packets");
            ui.end_row();

            ui.label("FDM / IMU");

            ui.label(if snapshot.enabled {
                "Streaming"
            } else {
                "Stopped"
            });

            ui.label(snapshot.tx_fdm_packets.to_string());

            ui.end_row();

            ui.label("Receiver / RC");

            ui.label(if snapshot.receiver_streaming {
                "Streaming"
            } else {
                "Stopped"
            });

            ui.label(snapshot.tx_rc_packets.to_string());

            ui.end_row();

            ui.label("Motor output");

            ui.label(if snapshot.connected() {
                "Receiving"
            } else {
                "Waiting"
            });

            ui.label(snapshot.rx_motor_packets.to_string());

            ui.end_row();
        });

    ui.add_space(8.0);

    match snapshot.last_rc_packet_age {
        Some(age) => {
            ui.label(format!("Last RC packet sent: {} ms ago", age.as_millis(),));
        }
        None => {
            ui.label("No RC packet has been sent yet.");
        }
    }

    match snapshot.last_motor_packet_age {
        Some(age) => {
            ui.label(format!("Last motor packet: {} ms ago", age.as_millis(),));
        }
        None => {
            ui.label("No motor packet received yet.");
        }
    }
}

fn show_motor_output(ui: &mut egui::Ui, snapshot: &SitlSnapshot) {
    ui.heading("Motor output from Betaflight");

    egui::Grid::new("betaflight_motor_grid")
        .num_columns(3)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Motor");
            ui.strong("Raw");
            ui.strong("Output");
            ui.end_row();

            for (index, command) in snapshot.motor_commands.iter().enumerate() {
                ui.label(format!("M{}", index + 1,));

                ui.label(format!("{command:+.5}",));

                ui.add(
                    egui::ProgressBar::new(command.clamp(0.0, 1.0))
                        .desired_width(300.0)
                        .text(format!("{:.1}%", command * 100.0,)),
                );

                ui.end_row();
            }
        });
}
