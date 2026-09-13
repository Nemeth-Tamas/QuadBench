use eframe::egui;
use quadbench_core::state::QuadState;

pub fn show(ui: &mut egui::Ui, state: &mut QuadState) {
    ui.heading("Fault Injection");
    ui.add_space(8.0);

    ui.group(|ui| {
        ui.strong("Receiver");

        ui.checkbox(&mut state.receiver.force_rx_loss, "Force RX loss");

        ui.label(
            "When enabled, QuadBench stops sending RC packets to Betaflight. \
             FDM / sensor data continues normally.",
        );

        ui.add_space(6.0);

        ui.label(format!(
            "Physical Pocket: {}",
            if state.receiver.connected {
                "Connected"
            } else {
                "Disconnected"
            },
        ));

        ui.label(format!(
            "Simulated failsafe: {}",
            if state.receiver.failsafe {
                "ACTIVE"
            } else {
                "clear"
            },
        ));

        ui.label(format!(
            "RC stream: {}",
            if state.receiver.force_rx_loss {
                "BLOCKED"
            } else if state.receiver.connected {
                "Allowed"
            } else {
                "No receiver"
            },
        ));
    });

    ui.add_space(16.0);

    ui.label(
        "More fault controls will land here for GPS, motors, ESCs, \
         sensors and battery simulation.",
    );
}
