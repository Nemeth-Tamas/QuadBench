use gilrs::{Axis, Button};

use crate::controller::ControllerSnapshot;

pub const POCKET_CHANNEL_COUNT: usize = 16;

const CHANNEL_MIN_US: f32 = 988.0;
const CHANNEL_MID_US: f32 = 1_500.0;
const CHANNEL_RANGE_US: f32 = 512.0;

const YAW_DEADBAND: f32 = 0.06;

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
    pub fn from_controller(snapshot: &ControllerSnapshot) -> Self {
        let roll = snapshot.axis_value(Axis::LeftStickX).unwrap_or(0.0);

        let pitch = snapshot.axis_value(Axis::LeftStickY).unwrap_or(0.0);

        let throttle = snapshot.axis_value(Axis::RightStickX).unwrap_or(-1.0);

        let yaw = centered_trigger_axis(
            mapped_button_value(snapshot, Button::LeftTrigger2, "LeftTrigger2").unwrap_or(0.5),
        );

        let arm = switch_position(
            mapped_button_value(snapshot, Button::RightTrigger2, "RightTrigger2").unwrap_or(0.0),
        );

        let sa = switch_position(
            mapped_button_value(snapshot, Button::RightTrigger, "RightTrigger").unwrap_or(0.0),
        );

        let sb = snap_three_position(snapshot.axis_value(Axis::RightStickY).unwrap_or(-1.0));

        let sc = switch_position(
            mapped_button_value(snapshot, Button::LeftTrigger, "LeftTrigger").unwrap_or(0.0),
        );

        let se =
            switch_position(mapped_button_value(snapshot, Button::West, "West").unwrap_or(0.0));

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

        channels_us[2] = 988;

        for control in &controls {
            channels_us[control.channel - 1] = control.pulse_us;
        }

        Self {
            controls,
            channels_us,
        }
    }
}

fn mapped_button_value(
    snapshot: &ControllerSnapshot,
    button: Button,
    logical_name: &str,
) -> Option<f32> {
    snapshot
        .raw_logical_value(logical_name)
        .or_else(|| snapshot.button_value(button))
}

fn centered_trigger_axis(value: f32) -> f32 {
    apply_center_deadband(trigger_to_bipolar(value), YAW_DEADBAND)
}

fn switch_position(value: f32) -> f32 {
    snap_three_position(trigger_to_bipolar(value))
}

fn trigger_to_bipolar(value: f32) -> f32 {
    (value.clamp(0.0, 1.0) * 2.0) - 1.0
}

fn snap_three_position(value: f32) -> f32 {
    let value = value.clamp(-1.0, 1.0);

    if value < -0.5 {
        -1.0
    } else if value > 0.5 {
        1.0
    } else {
        0.0
    }
}

fn apply_center_deadband(value: f32, deadband: f32) -> f32 {
    let value = value.clamp(-1.0, 1.0);
    let magnitude = value.abs();

    if magnitude <= deadband {
        return 0.0;
    }

    value.signum() * ((magnitude - deadband) / (1.0 - deadband)).clamp(0.0, 1.0)
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

fn normalized_to_us(value: f32) -> u16 {
    (CHANNEL_MID_US + value.clamp(-1.0, 1.0) * CHANNEL_RANGE_US)
        .round()
        .clamp(CHANNEL_MIN_US, CHANNEL_MID_US + CHANNEL_RANGE_US) as u16
}
