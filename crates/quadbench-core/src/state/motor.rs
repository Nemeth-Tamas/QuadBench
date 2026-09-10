use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotorFault {
    None,
    Stalled,
    Offline,
    Limited,
    Reversed,
}

impl MotorFault {
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "OK",
            Self::Stalled => "Stalled",
            Self::Offline => "Offline",
            Self::Limited => "Limited",
            Self::Reversed => "Reversed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorState {
    pub command: f32,
    pub rpm: f32,
    pub current_a: f32,
    pub temperature_c: f32,
    pub fault: MotorFault,
}

impl Default for MotorState {
    fn default() -> Self {
        Self {
            command: 0.0,
            rpm: 0.0,
            current_a: 0.0,
            temperature_c: 25.0,
            fault: MotorFault::None,
        }
    }
}
