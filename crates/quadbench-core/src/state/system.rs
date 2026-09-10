use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkState {
    Disconnected,
    Connecting,
    Connected,
    Faulted,
}

impl LinkState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting",
            Self::Connected => "Connected",
            Self::Faulted => "Fault",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub simulation_running: bool,
    pub simulation_rate_hz: u32,
    pub betaflight_link: LinkState,
    pub controller_link: LinkState,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            simulation_running: false,
            simulation_rate_hz: 1_000,
            betaflight_link: LinkState::Disconnected,
            controller_link: LinkState::Disconnected,
        }
    }
}
