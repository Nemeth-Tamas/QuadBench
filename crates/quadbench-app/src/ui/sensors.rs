use eframe::egui;
use quadbench_physics::PhysicsRuntime;

pub fn show(ui: &mut egui::Ui, physics: &PhysicsRuntime) {
    ui.heading("Virtual IMU / Physics");
    ui.add_space(8.0);

    let runtime = physics.telemetry();

    ui.group(|ui| {
        ui.strong("Physics worker");

        ui.label(format!(
            "State: {}",
            if runtime.enabled { "Running" } else { "Paused" },
        ));

        ui.label(format!(
            "Configured rate: {} Hz",
            runtime.configured_tick_rate_hz,
        ));

        ui.label(format!(
            "Measured rate: {:.1} Hz",
            runtime.measured_tick_rate_hz,
        ));

        ui.label(format!("Ticks: {}", runtime.tick_count,));

        ui.label(format!("Timing overruns: {}", runtime.overrun_count,));

        match runtime.last_tick_age {
            Some(age) => {
                ui.label(format!("Last tick: {} ms ago", age.as_millis(),));
            }
            None => {
                ui.label("No physics tick yet.");
            }
        }
    });

    ui.add_space(14.0);

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

        if ui.button("Respawn origin").clicked() {
            physics.reset_translation();
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

    ui.heading("Airframe model");

    egui::Grid::new("airframe_model_grid")
        .num_columns(2)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Mass");

            ui.label(format!("{:.3} kg", snapshot.parameters.mass_kg,));

            ui.end_row();

            ui.label("Motor radius");

            ui.label(format!(
                "{:.0} mm",
                snapshot.parameters.motor_radius_m * 1_000.0,
            ));

            ui.end_row();

            ui.label("Max thrust / motor");

            ui.label(format!("{:.2} N", snapshot.parameters.max_motor_thrust_n,));

            ui.end_row();

            ui.label("Weight");

            ui.label(format!("{:.2} N", snapshot.weight_n(),));

            ui.end_row();

            ui.label("Motor spool up");

            ui.label(format!(
                "{:.0} ms",
                snapshot.parameters.motor_spool_up_s * 1_000.0,
            ));

            ui.end_row();

            ui.label("Motor spool down");

            ui.label(format!(
                "{:.0} ms",
                snapshot.parameters.motor_spool_down_s * 1_000.0,
            ));

            ui.end_row();

            ui.label("Calculated hover");

            ui.label(format!(
                "{:.1}% motor",
                snapshot.hover_motor_command() * 100.0,
            ));

            ui.end_row();
        });

    ui.add_space(16.0);

    ui.heading("Flight state");

    egui::Grid::new("flight_state_grid")
        .num_columns(2)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Contact");

            ui.label(if snapshot.on_ground {
                "Ground"
            } else {
                "Airborne"
            });

            ui.end_row();

            ui.label("Total thrust");

            ui.label(format!("{:.2} N", snapshot.total_thrust_n,));

            ui.end_row();

            ui.label("World thrust");

            ui.label(format!(
                "E {:+.2}  N {:+.2}  U {:+.2} N",
                snapshot.world_thrust_n[0], snapshot.world_thrust_n[1], snapshot.world_thrust_n[2],
            ));

            ui.end_row();

            ui.label("Vertical thrust");

            ui.label(format!("{:.2} N", snapshot.vertical_thrust_n,));

            ui.end_row();

            ui.label("Thrust / weight");

            ui.label(format!("{:.2}:1", snapshot.thrust_to_weight(),));

            ui.end_row();

            ui.label("Altitude");

            ui.label(format!("{:.3} m", snapshot.position_enu_m[2],));

            ui.end_row();

            ui.label("Ground distance");

            ui.label(format!("{:.3} m", snapshot.ground_distance_m(),));

            ui.end_row();

            ui.label("Horizontal speed");

            ui.label(format!("{:.3} m/s", snapshot.horizontal_speed_mps(),));

            ui.end_row();

            ui.label("Total speed");

            ui.label(format!("{:.3} m/s", snapshot.total_speed_mps(),));

            ui.end_row();

            ui.label("Vertical speed");

            ui.label(format!("{:+.3} m/s", snapshot.velocity_enu_mps[2],));

            ui.end_row();

            ui.label("Vertical acceleration");

            ui.label(format!(
                "{:+.3} m/s2",
                snapshot.linear_acceleration_enu_mps2[2],
            ));

            ui.end_row();
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
        .num_columns(5)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Motor");
            ui.strong("Position");
            ui.strong("Target");
            ui.strong("Actual");
            ui.strong("Thrust");
            ui.end_row();

            for (index, position) in ["Rear right", "Front right", "Rear left", "Front left"]
                .into_iter()
                .enumerate()
            {
                ui.label(format!("M{}", index + 1,));

                ui.label(position);

                ui.label(format!("{:.1}%", snapshot.motor_commands[index] * 100.0,));

                ui.label(format!("{:.1}%", snapshot.motor_actual[index] * 100.0,));

                ui.label(format!("{:.2} N", snapshot.motor_thrust_n[index],));

                ui.end_row();
            }
        });

    ui.add_space(8.0);

    egui::Grid::new("sensor_motor_mix_grid")
        .num_columns(4)
        .spacing([16.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Axis");
            ui.strong("Mix");
            ui.strong("Torque");
            ui.strong("Acceleration");
            ui.end_row();

            for (axis, mix, torque, acceleration) in [
                (
                    "Roll",
                    snapshot.motor_mix[0],
                    snapshot.body_torque_nm[0],
                    acceleration[0],
                ),
                (
                    "Pitch",
                    snapshot.motor_mix[1],
                    snapshot.body_torque_nm[1],
                    acceleration[1],
                ),
                (
                    "Yaw",
                    snapshot.motor_mix[2],
                    snapshot.body_torque_nm[2],
                    acceleration[2],
                ),
            ] {
                ui.label(axis);

                ui.label(format!("{mix:+.4}",));

                ui.label(format!("{torque:+.4} N*m",));

                ui.label(format!("{acceleration:+.1} deg/s2",));

                ui.end_row();
            }
        });

    ui.add_space(14.0);

    ui.label(
        "Attitude is now integrated as a normalized \
         body-to-NWU quaternion, while motor commands \
         pass through finite spool-up / spool-down \
         dynamics before producing thrust and torque.",
    );
}
