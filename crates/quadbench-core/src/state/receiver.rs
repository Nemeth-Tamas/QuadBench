use serde::{Deserialize, Serialize};

pub const RECEIVER_CHANNEL_COUNT: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiverState {
    pub connected: bool,
    pub channels_us: [u16; RECEIVER_CHANNEL_COUNT],
    pub rssi_dbm: i16,
    pub link_quality: u8,
    pub packet_rate_hz: u16,
    pub failsafe: bool,
}

impl Default for ReceiverState {
    fn default() -> Self {
        let mut channels_us = [1_500; RECEIVER_CHANNEL_COUNT];

        // Temporary simulator default only.
        // Channel mapping becomes configurable when input support lands.
        channels_us[2] = 988;

        Self {
            connected: false,
            channels_us,
            rssi_dbm: -50,
            link_quality: 100,
            packet_rate_hz: 250,
            failsafe: false,
        }
    }
}
