use eframe::egui;
use quadbench_core::state::QuadState;
use quadbench_input::{ControllerDevice, ControllerSnapshot, PocketSnapshot};

pub fn show(
    ui: &mut egui::Ui,
    state: &QuadState,
    devices: &[ControllerDevice],
    snapshot: Option<&ControllerSnapshot>,
    pocket: Option<&PocketSnapshot>,
    input_error: Option<&str>,
) {
    ui.heading("Receiver");
    ui.add_space(8.0);

    if let Some(error) = input_error {
        ui.group(|ui| {
            ui.strong("Controller backend fault");
            ui.label(error);
        });

        return;
    }

    ui.group(|ui| {
        ui.strong("Virtual receiver");

        let receiver_state = if state.receiver.force_rx_loss {
            "RX loss forced"
        } else if state.receiver.connected {
            "Connected"
        } else {
            "Disconnected"
        };

        ui.label(format!("Receiver state: {receiver_state}",));

        ui.label(format!(
            "Failsafe: {}",
            if state.receiver.failsafe {
                "ACTIVE"
            } else {
                "clear"
            },
        ));

        ui.label(
            "Pocket input is translated to AETR + AUX channels \
             and streamed directly into Betaflight SITL.",
        );
    });

    ui.add_space(12.0);

    show_pocket_mapping(ui, pocket);

    ui.add_space(14.0);

    show_channel_monitor(ui, state);

    ui.add_space(14.0);

    ui.heading(format!("Detected USB controllers ({})", devices.len(),));

    ui.add_space(6.0);

    if devices.is_empty() {
        ui.label(
            "No compatible game controller \
             is currently detected.",
        );
    }

    for device in devices {
        ui.group(|ui| {
            ui.strong(format!("#{} — {}", device.id, device.name,));

            ui.label(format!("OS name: {}", device.os_name,));

            ui.label(format!("VID: {}", format_usb_id(device.vendor_id),));

            ui.label(format!("PID: {}", format_usb_id(device.product_id),));

            ui.label(format!("Mapping: {}", device.mapping_source,));
        });

        ui.add_space(6.0);
    }

    let Some(snapshot) = snapshot else {
        return;
    };

    ui.add_space(10.0);

    ui.heading("Active controller");

    ui.label(format!(
        "{} — device #{}",
        snapshot.device.name, snapshot.device.id,
    ));

    if let Some(last_event) = snapshot.last_event.as_deref() {
        ui.label(format!("Last event: {last_event}",));
    }

    ui.add_space(12.0);

    ui.heading("Mapped axes");

    if snapshot.axes.is_empty() {
        ui.label(
            "No standard mapped axes yet. \
             Raw events below are more important \
             for transmitter discovery.",
        );
    } else {
        egui::Grid::new("receiver_axis_grid")
            .num_columns(3)
            .spacing([12.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Axis");
                ui.strong("Value");
                ui.strong("Position");
                ui.end_row();

                for axis in &snapshot.axes {
                    ui.label(&axis.name);

                    ui.label(format!("{:+.4}", axis.value,));

                    ui.add(
                        egui::ProgressBar::new(((axis.value + 1.0) / 2.0).clamp(0.0, 1.0))
                            .desired_width(260.0),
                    );

                    ui.end_row();
                }
            });
    }

    ui.add_space(14.0);

    ui.heading("Mapped buttons");

    if snapshot.buttons.is_empty() {
        ui.label("No standard mapped buttons detected.");
    } else {
        egui::Grid::new("receiver_button_grid")
            .num_columns(3)
            .spacing([12.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Button");
                ui.strong("Value");
                ui.strong("Pressed");
                ui.end_row();

                for button in &snapshot.buttons {
                    ui.label(&button.name);

                    ui.label(format!("{:.3}", button.value,));

                    ui.label(if button.pressed { "YES" } else { "no" });

                    ui.end_row();
                }
            });
    }

    ui.add_space(14.0);

    ui.heading("Raw controls");

    ui.label(
        "Move one Pocket stick or switch at \
         a time. These raw codes let us discover \
         the transmitter even if Windows gives \
         it a stupid controller mapping.",
    );

    ui.add_space(6.0);

    if snapshot.raw_controls.is_empty() {
        ui.label("No raw input events received yet.");
    } else {
        egui::Grid::new("receiver_raw_grid")
            .num_columns(4)
            .spacing([12.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Code");
                ui.strong("Type");
                ui.strong("Mapped as");
                ui.strong("Value");
                ui.end_row();

                for control in &snapshot.raw_controls {
                    ui.label(&control.control);
                    ui.label(&control.kind);

                    ui.label(&control.logical_name);

                    ui.label(format!("{:+.4}", control.value,));

                    ui.end_row();
                }
            });
    }
}

fn show_pocket_mapping(ui: &mut egui::Ui, pocket: Option<&PocketSnapshot>) {
    ui.heading("RadioMaster Pocket");

    let Some(pocket) = pocket else {
        ui.label(
            "Pocket profile could not map the \
             required primary controls.",
        );

        return;
    };

    ui.label(
        "Physical Pocket controls translated \
         from the currently discovered Windows \
         controller mapping.",
    );

    ui.add_space(6.0);

    ui.group(|ui| {
        ui.strong("Yaw centering");

        ui.label(format!("Raw yaw input: {:.4}", pocket.yaw_raw,));

        ui.label(format!("Learned neutral: {:.4}", pocket.yaw_center_raw,));

        ui.label(format!("Corrected output: {} us", pocket.channels_us[3],));

        if pocket.yaw_calibrating {
            ui.add(
                egui::ProgressBar::new(pocket.yaw_calibration_progress.clamp(0.0, 1.0))
                    .desired_width(260.0)
                    .text("Learning neutral"),
            );

            ui.label("Leave the yaw stick released until calibration completes.");
        } else {
            ui.label("Neutral learned for this controller connection.");
        }
    });

    ui.add_space(8.0);

    egui::Grid::new("pocket_mapping_grid")
        .num_columns(5)
        .spacing([12.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Physical");
            ui.strong("Windows / gilrs");
            ui.strong("Channel");
            ui.strong("Normalized");
            ui.strong("Output");
            ui.end_row();

            for control in &pocket.controls {
                ui.label(control.name);
                ui.label(control.source);

                ui.label(format!("CH{}", control.channel,));

                ui.label(format!("{:+.4}", control.normalized,));

                ui.label(format!("{} µs", control.pulse_us,));

                ui.end_row();
            }
        });
}

fn show_channel_monitor(ui: &mut egui::Ui, state: &QuadState) {
    ui.heading("Receiver channels");

    ui.label(
        "AETR on CH1–CH4. SD is AUX1 so \
         it can become the Betaflight ARM \
         switch later.",
    );

    ui.add_space(6.0);

    egui::Grid::new("receiver_channel_grid")
        .num_columns(4)
        .spacing([12.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Channel");
            ui.strong("Function");
            ui.strong("Value");
            ui.strong("Position");
            ui.end_row();

            for (index, value) in state.receiver.channels_us.iter().enumerate() {
                ui.label(format!("CH{}", index + 1,));

                ui.label(channel_name(index));

                ui.label(format!("{value} µs",));

                let normalized = (f32::from(*value) - 988.0) / 1_024.0;

                ui.add(egui::ProgressBar::new(normalized.clamp(0.0, 1.0)).desired_width(240.0));

                ui.end_row();
            }
        });
}

fn channel_name(index: usize) -> &'static str {
    match index {
        0 => "Roll",
        1 => "Pitch",
        2 => "Throttle",
        3 => "Yaw",
        4 => "AUX1 / SD Arm",
        5 => "AUX2 / SA",
        6 => "AUX3 / SB",
        7 => "AUX4 / SC",
        8 => "AUX5 / SE",
        9 => "AUX6",
        10 => "AUX7",
        11 => "AUX8",
        12 => "AUX9",
        13 => "AUX10",
        14 => "AUX11",
        15 => "AUX12",
        _ => "Unknown",
    }
}

fn format_usb_id(value: Option<u16>) -> String {
    match value {
        Some(value) => {
            format!("{value:04X}")
        }
        None => "unknown".to_owned(),
    }
}
