use std::time::Duration;

use eframe::egui;
use quadbench_core::state::{LinkState, QuadState};
use quadbench_input::{ControllerDevice, ControllerInput, ControllerSnapshot};
use tracing::error;

use crate::ui::{self, UiPage};

pub struct QuadBenchApp {
    state: QuadState,
    selected_page: UiPage,
    controller_input: Option<ControllerInput>,
    controller_devices: Vec<ControllerDevice>,
    controller_snapshot: Option<ControllerSnapshot>,
    controller_error: Option<String>,
}

impl QuadBenchApp {
    pub fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {
        let (controller_input, controller_error) = match ControllerInput::new() {
            Ok(input) => (Some(input), None),
            Err(error) => {
                error!(
                    %error,
                    "failed to initialize controller input"
                );

                (None, Some(error.to_string()))
            }
        };

        let mut app = Self {
            state: QuadState::default(),
            selected_page: UiPage::Dashboard,
            controller_input,
            controller_devices: Vec::new(),
            controller_snapshot: None,
            controller_error,
        };

        app.poll_controller();

        app
    }

    fn poll_controller(&mut self) {
        let Some(input) = self.controller_input.as_mut() else {
            self.state.system.controller_link = LinkState::Faulted;

            self.controller_devices.clear();
            self.controller_snapshot = None;

            return;
        };

        input.poll();

        self.controller_devices = input.devices();

        self.controller_snapshot = input.snapshot();

        self.state.system.controller_link = if self.controller_snapshot.is_some() {
            LinkState::Connected
        } else {
            LinkState::Disconnected
        };
    }
}

impl eframe::App for QuadBenchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_controller();

        ui::top_bar::show(ui, &mut self.state);

        ui.separator();

        ui.horizontal(|ui| {
            ui::navigation::show(ui, &mut self.selected_page);

            ui.separator();

            ui.vertical(|ui| {
                ui.set_min_width(700.0);

                match self.selected_page {
                    UiPage::Dashboard => {
                        ui::dashboard::show(ui, &self.state);
                    }
                    UiPage::Receiver => {
                        ui::receiver::show(
                            ui,
                            &self.state,
                            &self.controller_devices,
                            self.controller_snapshot.as_ref(),
                            self.controller_error.as_deref(),
                        );
                    }
                    page => {
                        ui::show_placeholder(ui, page);
                    }
                }
            });
        });

        ui.separator();

        ui::status_bar::show(ui, &self.state);

        ui.ctx().request_repaint_after(Duration::from_millis(16));
    }
}
