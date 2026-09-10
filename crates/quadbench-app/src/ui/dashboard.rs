use eframe::egui;
use quadbench_core::state::QuadState;

pub fn show(ui: &mut egui::Ui, state: &QuadState) {
    ui.heading("Dashboard");
    ui.add_space(8.0);

    show_hardware_summary(ui, state);

    ui.add_space(16.0);

    ui.heading("Motors");
    ui.add_space(8.0);

    show_motors(ui, state);
}

fn show_hardware_summary(ui: &mut egui::Ui, state: &QuadState) {
    ui.columns(4, |columns| {
        columns[0].group(|ui| {
            ui.strong("Battery");

            ui.label(format!("{:.2} V", state.battery.voltage_v,));

            ui.label(format!("{:.2} V / cell", state.battery.cell_voltage_v(),));

            ui.label(format!("{:.1} A", state.battery.current_a,));

            ui.label(format!("{:.0}%", state.battery.state_of_charge * 100.0,));
        });

        columns[1].group(|ui| {
            ui.strong("Receiver");

            let connection = if state.receiver.connected {
                "Connected"
            } else {
                "Disconnected"
            };

            ui.label(connection);

            ui.label(format!("LQ: {}%", state.receiver.link_quality,));

            ui.label(format!("RSSI: {} dBm", state.receiver.rssi_dbm,));

            ui.label(format!("{} Hz", state.receiver.packet_rate_hz,));
        });

        columns[2].group(|ui| {
            ui.strong("GPS");

            let connection = if state.gps.connected {
                "Connected"
            } else {
                "Disconnected"
            };

            ui.label(connection);

            ui.label(format!("Fix: {}", state.gps.fix.label(),));

            ui.label(format!("Satellites: {}", state.gps.satellites,));

            ui.label(format!("Altitude: {:.1} m", state.gps.altitude_m,));
        });

        columns[3].group(|ui| {
            ui.strong("Flight Controller");

            ui.label(format!("SITL: {}", state.system.betaflight_link.label(),));

            let simulation = if state.system.simulation_running {
                "Running"
            } else {
                "Stopped"
            };

            ui.label(format!("Simulation: {simulation}",));

            ui.label(format!(
                "Tick target: {} Hz",
                state.system.simulation_rate_hz,
            ));
        });
    });
}

fn show_motors(ui: &mut egui::Ui, state: &QuadState) {
    egui::Grid::new("motor_grid")
        .num_columns(5)
        .spacing([12.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Motor");
            ui.strong("Command");
            ui.strong("RPM");
            ui.strong("Current");
            ui.strong("Status");
            ui.end_row();

            for (index, motor) in state.motors.iter().enumerate() {
                ui.label(format!("M{}", index + 1,));

                ui.add(
                    egui::ProgressBar::new(motor.command.clamp(0.0, 1.0))
                        .desired_width(220.0)
                        .text(format!("{:.1}%", motor.command * 100.0,)),
                );

                ui.label(format!("{:.0}", motor.rpm,));

                ui.label(format!("{:.1} A", motor.current_a,));

                ui.label(motor.fault.label());

                ui.end_row();
            }
        });
}
