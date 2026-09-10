use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub connected: bool,
    pub cell_count: u8,
    pub capacity_mah: u32,
    pub consumed_mah: f32,
    pub voltage_v: f32,
    pub current_a: f32,
    pub state_of_charge: f32,
}

impl Default for BatteryState {
    fn default() -> Self {
        Self {
            connected: true,
            cell_count: 6,
            capacity_mah: 1_500,
            consumed_mah: 0.0,
            voltage_v: 25.2,
            current_a: 0.0,
            state_of_charge: 1.0,
        }
    }
}

impl BatteryState {
    pub fn cell_voltage_v(&self) -> f32 {
        if self.cell_count == 0 {
            return 0.0;
        }

        self.voltage_v / f32::from(self.cell_count)
    }
}
