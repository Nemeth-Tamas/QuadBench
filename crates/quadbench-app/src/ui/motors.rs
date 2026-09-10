use eframe::egui;
use quadbench_core::state::QuadState;

pub fn show(ui: &mut egui::Ui, state: &QuadState) {
    ui.heading("Motors / ESC");
    ui.add_space(8.0);

    ui.label(
        "Commands below are the normalized \
         motor outputs received from Betaflight SITL.",
    );

    ui.add_space(12.0);

    for (index, motor) in state.motors.iter().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("Motor {}", index + 1,));

                ui.label(format!("{:.5}", motor.command,));

                ui.label(format!("{:.1}%", motor.command * 100.0,));
            });

            ui.add(
                egui::ProgressBar::new(motor.command.clamp(0.0, 1.0))
                    .desired_width(500.0)
                    .text(format!("{:.1}%", motor.command * 100.0,)),
            );

            ui.horizontal(|ui| {
                ui.label(format!("RPM: {:.0}", motor.rpm,));

                ui.separator();

                ui.label(format!("Current: {:.1} A", motor.current_a,));

                ui.separator();

                ui.label(format!("Temperature: {:.1} °C", motor.temperature_c,));

                ui.separator();

                ui.label(format!("Status: {}", motor.fault.label(),));
            });
        });

        ui.add_space(8.0);
    }

    ui.add_space(10.0);

    ui.label(
        "RPM/current/temperature remain simulated \
         hardware values. The ESC model will derive \
         those from Betaflight's motor commands later.",
    );
}
