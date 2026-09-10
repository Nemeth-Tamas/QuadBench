use gilrs::{Axis, Button};

use crate::controller::ControllerSnapshot;

pub const POCKET_CHANNEL_COUNT: usize = 16;

const CHANNEL_MIN_US: f32 = 988.0;
const CHANNEL_MID_US: f32 = 1_500.0;
const CHANNEL_RANGE_US: f32 = 512.0;

#[derive(Debug, Clone)]
pub struct PocketControlSnapshot {
    pub name: &'static str,
    pub source: &'static str,
    pub channel: usize,
    pub normalized: f32,
    pub pulse_us: u16,
}

#[derive(Debug, Clone)]
pub struct PocketSnapshot {
    pub controls: Vec<PocketControlSnapshot>,
    pub channels_us: [u16; POCKET_CHANNEL_COUNT],
}

impl PocketSnapshot {
    pub fn from_controller(snapshot: &ControllerSnapshot) -> Option<Self> {
        let roll = snapshot.axis_value(Axis::LeftStickX)?;

        let pitch = snapshot.axis_value(Axis::LeftStickY)?;

        let throttle = snapshot.axis_value(Axis::RightStickX)?;

        let yaw = button_bipolar(snapshot.button_value(Button::LeftTrigger2)?);

        let arm = button_bipolar(snapshot.button_value(Button::RightTrigger2).unwrap_or(0.0));

        let sa = button_bipolar(snapshot.button_value(Button::RightTrigger).unwrap_or(0.0));

        let sb = snapshot.axis_value(Axis::RightStickY).unwrap_or(-1.0);

        let sc = button_bipolar(snapshot.button_value(Button::LeftTrigger).unwrap_or(0.0));

        let se = button_bipolar(snapshot.button_value(Button::West).unwrap_or(0.0));

        let controls = vec![
            control("Roll", "Right stick L/R → LeftStickX", 1, roll),
            control("Pitch", "Right stick U/D → LeftStickY", 2, pitch),
            control("Throttle", "Left stick U/D → RightStickX", 3, throttle),
            control("Yaw", "Left stick L/R → LeftTrigger2", 4, yaw),
            control("Arm / SD", "SD → RightTrigger2", 5, arm),
            control("SA", "SA → RightTrigger", 6, sa),
            control("SB", "SB → RightStickY", 7, sb),
            control("SC", "SC → LeftTrigger", 8, sc),
            control("SE", "SE → West", 9, se),
        ];

        let mut channels_us = [1_500; POCKET_CHANNEL_COUNT];

        for control in &controls {
            channels_us[control.channel - 1] = control.pulse_us;
        }

        Some(Self {
            controls,
            channels_us,
        })
    }
}

fn control(
    name: &'static str,
    source: &'static str,
    channel: usize,
    normalized: f32,
) -> PocketControlSnapshot {
    PocketControlSnapshot {
        name,
        source,
        channel,
        normalized,
        pulse_us: normalized_to_us(normalized),
    }
}

fn button_bipolar(value: f32) -> f32 {
    (value.clamp(0.0, 1.0) * 2.0) - 1.0
}

fn normalized_to_us(value: f32) -> u16 {
    (CHANNEL_MID_US + value.clamp(-1.0, 1.0) * CHANNEL_RANGE_US)
        .round()
        .clamp(CHANNEL_MIN_US, CHANNEL_MID_US + CHANNEL_RANGE_US) as u16
}
