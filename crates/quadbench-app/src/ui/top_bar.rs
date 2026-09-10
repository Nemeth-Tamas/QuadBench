use eframe::egui;
use quadbench_core::state::QuadState;

pub fn show(ui: &mut egui::Ui, state: &mut QuadState) {
    ui.horizontal(|ui| {
        ui.heading("QuadBench");

        ui.separator();

        let button_text = if state.system.simulation_running {
            "Stop Simulation"
        } else {
            "Start Simulation"
        };

        if ui.button(button_text).clicked() {
            state.system.simulation_running = !state.system.simulation_running;
        }

        ui.label(format!("Target: {} Hz", state.system.simulation_rate_hz,));
    });
}
