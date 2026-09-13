use eframe::egui;
use quadbench_physics::PhysicsModel;

pub fn show(ui: &mut egui::Ui, physics: &mut PhysicsModel) {
    ui.heading("Virtual IMU / Physics");
    ui.add_space(8.0);

    let snapshot = physics.snapshot();

    let attitude = snapshot.attitude_deg();

    let rates = snapshot.angular_velocity_deg_s();

    ui.group(|ui| {
        ui.strong("Attitude");

        egui::Grid::new("sensor_attitude_grid")
            .num_columns(3)
            .spacing([16.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Axis");
                ui.strong("Angle");
                ui.strong("Rate");
                ui.end_row();

                for (name, angle, rate) in [
                    ("Roll", attitude[0], rates[0]),
                    ("Pitch", attitude[1], rates[1]),
                    ("Yaw", attitude[2], rates[2]),
                ] {
                    ui.label(name);

                    ui.label(format!("{angle:+.2} deg",));

                    ui.label(format!("{rate:+.2} deg/s",));

                    ui.end_row();
                }
            });
    });

    ui.add_space(14.0);

    ui.heading("Manual attitude");

    let mut edited_attitude = attitude;

    let mut changed = false;

    changed |= ui
        .add(
            egui::Slider::new(&mut edited_attitude[0], -60.0..=60.0)
                .text("Roll")
                .suffix(" deg"),
        )
        .changed();

    changed |= ui
        .add(
            egui::Slider::new(&mut edited_attitude[1], -60.0..=60.0)
                .text("Pitch")
                .suffix(" deg"),
        )
        .changed();

    changed |= ui
        .add(
            egui::Slider::new(&mut edited_attitude[2], -180.0..=180.0)
                .text("Yaw")
                .suffix(" deg"),
        )
        .changed();

    if changed {
        physics.set_attitude_deg(edited_attitude[0], edited_attitude[1], edited_attitude[2]);
    }

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("Reset level").clicked() {
            physics.reset_attitude();
        }

        if ui.button("Reset all physics").clicked() {
            physics.reset_all();
        }
    });

    ui.add_space(16.0);

    ui.heading("Angular disturbance");

    ui.label(
        "These inject angular velocity so the \
         virtual quad visibly rotates and then \
         the rate decays.",
    );

    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if ui.button("Roll left").clicked() {
            physics.add_angular_velocity_deg_s([-120.0, 0.0, 0.0]);
        }

        if ui.button("Roll right").clicked() {
            physics.add_angular_velocity_deg_s([120.0, 0.0, 0.0]);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Pitch up").clicked() {
            physics.add_angular_velocity_deg_s([0.0, -120.0, 0.0]);
        }

        if ui.button("Pitch down").clicked() {
            physics.add_angular_velocity_deg_s([0.0, 120.0, 0.0]);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Yaw left").clicked() {
            physics.add_angular_velocity_deg_s([0.0, 0.0, -120.0]);
        }

        if ui.button("Yaw right").clicked() {
            physics.add_angular_velocity_deg_s([0.0, 0.0, 120.0]);
        }
    });

    ui.add_space(16.0);

    ui.heading("Inertial state");

    egui::Grid::new("sensor_inertial_grid")
        .num_columns(4)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("");
            ui.strong("X / East");
            ui.strong("Y / North");
            ui.strong("Z / Up");
            ui.end_row();

            ui.label("Acceleration");

            for value in snapshot.linear_acceleration_enu_mps2 {
                ui.label(format!("{value:+.3} m/s2",));
            }

            ui.end_row();

            ui.label("Velocity");

            for value in snapshot.velocity_enu_mps {
                ui.label(format!("{value:+.3} m/s",));
            }

            ui.end_row();

            ui.label("Position");

            for value in snapshot.position_enu_m {
                ui.label(format!("{value:+.3} m",));
            }

            ui.end_row();
        });

    ui.add_space(16.0);

    ui.heading("Motor-driven angular plant");

    let acceleration = snapshot.angular_acceleration_deg_s2();

    egui::Grid::new("sensor_motor_grid")
        .num_columns(3)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Motor");
            ui.strong("Position");
            ui.strong("Command");
            ui.end_row();

            for (index, position) in ["Rear right", "Front right", "Rear left", "Front left"]
                .into_iter()
                .enumerate()
            {
                ui.label(format!("M{}", index + 1,));

                ui.label(position);

                ui.label(format!("{:.1}%", snapshot.motor_commands[index] * 100.0,));

                ui.end_row();
            }
        });

    ui.add_space(8.0);

    egui::Grid::new("sensor_motor_mix_grid")
        .num_columns(3)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Axis");
            ui.strong("Mix");
            ui.strong("Acceleration");
            ui.end_row();

            for (axis, mix, acceleration) in [
                ("Roll", snapshot.motor_mix[0], acceleration[0]),
                ("Pitch", snapshot.motor_mix[1], acceleration[1]),
                ("Yaw", snapshot.motor_mix[2], acceleration[2]),
            ] {
                ui.label(axis);

                ui.label(format!("{mix:+.4}",));

                ui.label(format!("{acceleration:+.1} deg/s2",));

                ui.end_row();
            }
        });

    ui.add_space(14.0);

    ui.label(
        "Betaflight motor output now drives \
         angular physics and feeds the resulting \
         attitude and gyro state back into SITL. \
         Translational thrust and full physical \
         mass / inertia modelling come next.",
    );
}
