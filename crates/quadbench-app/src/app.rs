use std::time::{Duration, Instant};

use eframe::egui;
use quadbench_betaflight::{
    ConfiguratorProxy, ConfiguratorProxyConfig, ConfiguratorProxySnapshot, FdmState, SitlBridge,
    SitlConfig, SitlSnapshot,
};
use quadbench_core::state::{LinkState, QuadState};
use quadbench_input::{
    ControllerDevice, ControllerInput, ControllerSnapshot, PocketProfile, PocketSnapshot,
};
use quadbench_physics::PhysicsModel;
use tracing::error;

use crate::ui::{self, UiPage};

pub struct QuadBenchApp {
    state: QuadState,
    physics: PhysicsModel,
    last_physics_step: Instant,
    selected_page: UiPage,
    controller_input: Option<ControllerInput>,
    controller_devices: Vec<ControllerDevice>,
    controller_snapshot: Option<ControllerSnapshot>,
    pocket_profile: PocketProfile,
    pocket_snapshot: Option<PocketSnapshot>,
    controller_error: Option<String>,
    sitl_bridge: Option<SitlBridge>,
    sitl_snapshot: Option<SitlSnapshot>,
    sitl_error: Option<String>,
    configurator_proxy: Option<ConfiguratorProxy>,
    configurator_proxy_snapshot: Option<ConfiguratorProxySnapshot>,
    configurator_proxy_error: Option<String>,
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

        let sitl_config = SitlConfig::default();

        let (sitl_bridge, sitl_error) = match SitlBridge::spawn(sitl_config) {
            Ok(bridge) => (Some(bridge), None),
            Err(error) => {
                error!(
                    %error,
                    "failed to initialize Betaflight SITL bridge"
                );

                (None, Some(error.to_string()))
            }
        };

        let proxy_config = ConfiguratorProxyConfig {
            target_host: sitl_config.target_host,
            ..ConfiguratorProxyConfig::default()
        };

        let (configurator_proxy, configurator_proxy_error) =
            match ConfiguratorProxy::spawn(proxy_config) {
                Ok(proxy) => (Some(proxy), None),
                Err(error) => {
                    error!(
                        %error,
                        "failed to initialize Configurator proxy"
                    );

                    (None, Some(error.to_string()))
                }
            };

        let mut app = Self {
            state: QuadState::default(),
            physics: PhysicsModel::default(),
            last_physics_step: Instant::now(),
            selected_page: UiPage::Dashboard,
            controller_input,
            controller_devices: Vec::new(),
            controller_snapshot: None,
            pocket_profile: PocketProfile::default(),
            pocket_snapshot: None,
            controller_error,
            sitl_bridge,
            sitl_snapshot: None,
            sitl_error,
            configurator_proxy,
            configurator_proxy_snapshot: None,
            configurator_proxy_error,
        };

        app.poll_controller();
        app.poll_sitl();
        app.poll_configurator_proxy();

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

        let was_connected = self.controller_snapshot.is_some();

        self.controller_snapshot = input.snapshot();

        let is_connected = self.controller_snapshot.is_some();

        if is_connected && !was_connected {
            self.pocket_profile.reset_yaw_calibration();
        }

        self.pocket_snapshot = match self.controller_snapshot.as_ref() {
            Some(snapshot) => Some(self.pocket_profile.snapshot(snapshot)),
            None => None,
        };

        if let Some(pocket) = self.pocket_snapshot.as_ref() {
            self.state.receiver.channels_us = pocket.channels_us;
        }

        self.state.receiver.connected = is_connected;

        self.state.system.controller_link = if is_connected {
            LinkState::Connected
        } else {
            LinkState::Disconnected
        };
    }

    fn step_physics(&mut self) {
        let now = Instant::now();

        let dt_seconds = now.duration_since(self.last_physics_step).as_secs_f64();

        self.last_physics_step = now;

        if self.state.system.simulation_running {
            self.physics.step(dt_seconds);
        }
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

        let physics = self.physics.snapshot();

        bridge.set_fdm_state(FdmState::from_body_kinematics(
            physics.attitude_rad,
            physics.angular_velocity_rad_s,
            physics.linear_acceleration_enu_mps2,
            physics.velocity_enu_mps,
            physics.position_enu_m,
        ));

        let receiver_streaming =
            self.state.receiver.connected && !self.state.receiver.force_rx_loss;

        self.state.receiver.failsafe = self.state.system.simulation_running && !receiver_streaming;

        bridge.set_receiver_enabled(receiver_streaming);

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

    fn poll_configurator_proxy(&mut self) {
        self.configurator_proxy_snapshot = self
            .configurator_proxy
            .as_ref()
            .map(ConfiguratorProxy::snapshot);
    }
}

impl eframe::App for QuadBenchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_controller();
        self.step_physics();
        self.poll_sitl();
        self.poll_configurator_proxy();

        ui::shell::show(
            ui,
            &mut self.state,
            &mut self.physics,
            &mut self.selected_page,
            &self.controller_devices,
            self.controller_snapshot.as_ref(),
            self.pocket_snapshot.as_ref(),
            self.controller_error.as_deref(),
            self.sitl_snapshot.as_ref(),
            self.sitl_error.as_deref(),
            self.configurator_proxy_snapshot.as_ref(),
            self.configurator_proxy_error.as_deref(),
        );

        ui.ctx().request_repaint_after(Duration::from_millis(16));
    }
}
