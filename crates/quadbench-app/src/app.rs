use std::time::Duration;

use eframe::egui;
use quadbench_betaflight::{SitlBridge, SitlConfig, SitlSnapshot};
use quadbench_core::state::{LinkState, QuadState};
use quadbench_input::{ControllerDevice, ControllerInput, ControllerSnapshot, PocketSnapshot};
use tracing::error;

use crate::ui::{self, UiPage};

pub struct QuadBenchApp {
    state: QuadState,
    selected_page: UiPage,
    controller_input: Option<ControllerInput>,
    controller_devices: Vec<ControllerDevice>,
    controller_snapshot: Option<ControllerSnapshot>,
    pocket_snapshot: Option<PocketSnapshot>,
    controller_error: Option<String>,
    sitl_bridge: Option<SitlBridge>,
    sitl_snapshot: Option<SitlSnapshot>,
    sitl_error: Option<String>,
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

        let (sitl_bridge, sitl_error) = match SitlBridge::spawn(SitlConfig::default()) {
            Ok(bridge) => (Some(bridge), None),
            Err(error) => {
                error!(
                    %error,
                    "failed to initialize Betaflight SITL bridge"
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
            pocket_snapshot: None,
            controller_error,
            sitl_bridge,
            sitl_snapshot: None,
            sitl_error,
        };

        app.poll_controller();
        app.poll_sitl();

        app
    }

    fn poll_controller(&mut self) {
        let Some(input) = self.controller_input.as_mut() else {
            self.state.system.controller_link = LinkState::Faulted;

            self.controller_devices.clear();
            self.controller_snapshot = None;
            self.pocket_snapshot = None;

            self.state.receiver.connected = false;

            return;
        };

        input.poll();

        self.controller_devices = input.devices();

        self.controller_snapshot = input.snapshot();

        self.pocket_snapshot = self
            .controller_snapshot
            .as_ref()
            .and_then(PocketSnapshot::from_controller);

        if let Some(pocket) = self.pocket_snapshot.as_ref() {
            self.state.receiver.connected = true;

            self.state.receiver.channels_us = pocket.channels_us;
        } else {
            self.state.receiver.connected = false;
        }

        self.state.system.controller_link = if self.controller_snapshot.is_some() {
            LinkState::Connected
        } else {
            LinkState::Disconnected
        };
    }

    fn poll_sitl(&mut self) {
        let Some(bridge) = self.sitl_bridge.as_ref() else {
            self.state.system.betaflight_link = LinkState::Faulted;

            self.sitl_snapshot = None;

            for motor in &mut self.state.motors {
                motor.command = 0.0;
            }

            return;
        };

        bridge.set_enabled(self.state.system.simulation_running);

        bridge.set_receiver_enabled(self.state.receiver.connected);

        bridge.set_channels(self.state.receiver.channels_us);

        let snapshot = bridge.snapshot();

        let connected = snapshot.connected();

        self.state.system.betaflight_link = if snapshot.last_error.is_some() {
            LinkState::Faulted
        } else if !snapshot.enabled {
            LinkState::Disconnected
        } else if connected {
            LinkState::Connected
        } else {
            LinkState::Connecting
        };

        if connected {
            for (motor, command) in self
                .state
                .motors
                .iter_mut()
                .zip(snapshot.motor_commands.iter().copied())
            {
                motor.command = command.clamp(0.0, 1.0);
            }
        } else {
            for motor in &mut self.state.motors {
                motor.command = 0.0;
            }
        }

        self.sitl_snapshot = Some(snapshot);
    }
}

impl eframe::App for QuadBenchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_controller();
        self.poll_sitl();

        ui::shell::show(
            ui,
            &mut self.state,
            &mut self.selected_page,
            &self.controller_devices,
            self.controller_snapshot.as_ref(),
            self.pocket_snapshot.as_ref(),
            self.controller_error.as_deref(),
            self.sitl_snapshot.as_ref(),
            self.sitl_error.as_deref(),
        );

        ui.ctx().request_repaint_after(Duration::from_millis(16));
    }
}
