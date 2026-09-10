mod battery;
mod gps;
mod motor;
mod receiver;
mod system;

pub use battery::BatteryState;
pub use gps::{GpsFix, GpsState};
pub use motor::{MotorFault, MotorState};
pub use receiver::ReceiverState;
pub use system::{LinkState, SystemState};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadState {
    pub system: SystemState,
    pub receiver: ReceiverState,
    pub battery: BatteryState,
    pub gps: GpsState,
    pub motors: [MotorState; 4],
}

impl Default for QuadState {
    fn default() -> Self {
        Self {
            system: SystemState::default(),
            receiver: ReceiverState::default(),
            battery: BatteryState::default(),
            gps: GpsState::default(),
            motors: std::array::from_fn(|_| MotorState::default()),
        }
    }
}
