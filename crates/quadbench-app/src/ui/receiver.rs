use eframe::egui;
use quadbench_core::state::QuadState;
use quadbench_input::{ControllerDevice, ControllerSnapshot};

pub fn show(
    ui: &mut egui::Ui,
    state: &QuadState,
    devices: &[ControllerDevice],
    snapshot: Option<&ControllerSnapshot>,
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

        let receiver_state = if state.receiver.connected {
            "Connected"
        } else {
            "Not bridged yet"
        };

        ui.label(format!("Receiver state: {receiver_state}",));

        ui.label(
            "USB controller detection is live. \
             Channel mapping and CRSF bridging \
             come next.",
        );
    });

    ui.add_space(12.0);

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

fn format_usb_id(value: Option<u16>) -> String {
    match value {
        Some(value) => {
            format!("{value:04X}")
        }
        None => "unknown".to_owned(),
    }
}
