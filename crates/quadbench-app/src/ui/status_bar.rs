use eframe::egui;
use quadbench_core::state::QuadState;

pub fn show(ui: &mut egui::Ui, state: &QuadState) {
    ui.horizontal(|ui| {
        let simulation_state = if state.system.simulation_running {
            "Running"
        } else {
            "Stopped"
        };

        ui.label(format!("Simulation: {simulation_state}",));

        ui.separator();

        ui.label(format!(
            "Betaflight: {}",
            state.system.betaflight_link.label(),
        ));

        ui.separator();

        ui.label(format!(
            "Controller: {}",
            state.system.controller_link.label(),
        ));

        ui.separator();

        let receiver_state = if state.receiver.connected {
            "Connected"
        } else {
            "Disconnected"
        };

        ui.label(format!("Receiver: {receiver_state}",));
    });
}
