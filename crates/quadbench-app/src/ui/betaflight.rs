use eframe::egui;
use quadbench_betaflight::SitlSnapshot;
use quadbench_core::state::QuadState;

pub fn show(
    ui: &mut egui::Ui,
    state: &QuadState,
    snapshot: Option<&SitlSnapshot>,
    bridge_error: Option<&str>,
) {
    ui.heading("Betaflight SITL");
    ui.add_space(8.0);

    if let Some(error) = bridge_error {
        ui.group(|ui| {
            ui.strong("Bridge initialization failed");
            ui.label(error);
        });

        return;
    }

    let Some(snapshot) = snapshot else {
        ui.label("SITL bridge has no runtime state.");

        return;
    };

    ui.group(|ui| {
        ui.strong("Connection");

        ui.label(format!("State: {}", state.system.betaflight_link.label(),));

        ui.label(format!("Target host: {}", snapshot.config.target_host,));

        ui.label(format!("Motor output: UDP {}", snapshot.config.motor_port,));

        ui.label(format!("FDM input: UDP {}", snapshot.config.fdm_port,));

        ui.label(format!("RC input: UDP {}", snapshot.config.rc_port,));

        ui.label(
            "MSP / Configurator: TCP 5761 \
             directly on Betaflight SITL",
        );
    });

    ui.add_space(12.0);

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

    ui.add_space(12.0);

    ui.heading("Last motor packet");

    match snapshot.last_motor_packet_age {
        Some(age) => {
            ui.label(format!("{} ms ago", age.as_millis(),));
        }
        None => {
            ui.label("No motor packet received yet.");
        }
    }

    if let Some(error) = snapshot.last_error.as_deref() {
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.strong("Last transport error");
            ui.label(error);
        });
    }

    ui.add_space(12.0);

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

    ui.add_space(14.0);

    ui.group(|ui| {
        ui.strong("Current simulator plant");

        ui.label(
            "QuadBench currently sends a stationary \
             level IMU/GPS state. Real flight physics \
             comes after the SITL transport is proven.",
        );

        ui.label(
            "Start Simulation enables FDM streaming. \
             RC streaming only runs while the Pocket \
             is connected.",
        );
    });
}
