use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpsFix {
    None,
    TwoDimensional,
    ThreeDimensional,
}

impl GpsFix {
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "No fix",
            Self::TwoDimensional => "2D",
            Self::ThreeDimensional => "3D",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsState {
    pub connected: bool,
    pub fix: GpsFix,
    pub satellites: u8,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_m: f32,
    pub ground_speed_mps: f32,
    pub heading_deg: f32,
}

impl Default for GpsState {
    fn default() -> Self {
        Self {
            connected: false,
            fix: GpsFix::None,
            satellites: 0,
            latitude_deg: 0.0,
            longitude_deg: 0.0,
            altitude_m: 0.0,
            ground_speed_mps: 0.0,
            heading_deg: 0.0,
        }
    }
}
