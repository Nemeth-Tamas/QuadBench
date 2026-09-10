use std::{collections::BTreeMap, error::Error, fmt};

use gilrs::{Axis, Button, EventType, Gilrs, GilrsBuilder};
use tracing::{debug, info};

const AXES: [Axis; 8] = [
    Axis::LeftStickX,
    Axis::LeftStickY,
    Axis::LeftZ,
    Axis::RightStickX,
    Axis::RightStickY,
    Axis::RightZ,
    Axis::DPadX,
    Axis::DPadY,
];

const BUTTONS: [Button; 19] = [
    Button::South,
    Button::East,
    Button::North,
    Button::West,
    Button::C,
    Button::Z,
    Button::LeftTrigger,
    Button::LeftTrigger2,
    Button::RightTrigger,
    Button::RightTrigger2,
    Button::Select,
    Button::Start,
    Button::Mode,
    Button::LeftThumb,
    Button::RightThumb,
    Button::DPadUp,
    Button::DPadDown,
    Button::DPadLeft,
    Button::DPadRight,
];

#[derive(Debug)]
pub enum InputError {
    Initialization(String),
}

impl fmt::Display for InputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialization(message) => {
                write!(
                    formatter,
                    "controller input initialization failed: {message}",
                )
            }
        }
    }
}

impl Error for InputError {}

#[derive(Debug, Clone)]
pub struct ControllerDevice {
    pub id: usize,
    pub name: String,
    pub os_name: String,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub mapping_source: String,
}

#[derive(Debug, Clone)]
pub struct AxisSnapshot {
    pub name: String,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct ButtonSnapshot {
    pub name: String,
    pub value: f32,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub struct RawControlSnapshot {
    pub control: String,
    pub kind: String,
    pub logical_name: String,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct ControllerSnapshot {
    pub device: ControllerDevice,
    pub axes: Vec<AxisSnapshot>,
    pub buttons: Vec<ButtonSnapshot>,
    pub raw_controls: Vec<RawControlSnapshot>,
    pub last_event: Option<String>,
}

impl ControllerSnapshot {
    pub(crate) fn axis_value(&self, axis: Axis) -> Option<f32> {
        let name = format!("{axis:?}");

        self.axes
            .iter()
            .find(|snapshot| snapshot.name == name)
            .map(|snapshot| snapshot.value)
    }

    pub(crate) fn button_value(&self, button: Button) -> Option<f32> {
        let name = format!("{button:?}");

        self.buttons
            .iter()
            .find(|snapshot| snapshot.name == name)
            .map(|snapshot| snapshot.value)
    }

    pub(crate) fn raw_logical_value(&self, logical_name: &str) -> Option<f32> {
        self.raw_controls
            .iter()
            .find(|snapshot| {
                snapshot.logical_name == logical_name && snapshot.kind == "Analog button"
            })
            .or_else(|| {
                self.raw_controls
                    .iter()
                    .find(|snapshot| snapshot.logical_name == logical_name)
            })
            .map(|snapshot| snapshot.value)
    }
}

pub struct ControllerInput {
    gilrs: Gilrs,
    selected_id: Option<usize>,
    raw_controls: BTreeMap<(usize, String), RawControlSnapshot>,
    last_event: Option<String>,
}

impl ControllerInput {
    pub fn new() -> Result<Self, InputError> {
        let gilrs = GilrsBuilder::new()
            .with_force_feedback(false)
            .build()
            .map_err(|error| InputError::Initialization(error.to_string()))?;

        let selected_id = gilrs.gamepads().next().map(|(id, _)| usize::from(id));

        let input = Self {
            gilrs,
            selected_id,
            raw_controls: BTreeMap::new(),
            last_event: None,
        };

        for device in input.devices() {
            info!(
                id = device.id,
                name = %device.name,
                os_name = %device.os_name,
                vendor_id = ?device.vendor_id,
                product_id = ?device.product_id,
                "controller detected"
            );
        }

        Ok(input)
    }

    pub fn poll(&mut self) {
        while let Some(event) = self.gilrs.next_event() {
            let device_id = usize::from(event.id);

            debug!(
                device_id,
                event = ?event.event,
                "controller event"
            );

            self.last_event = Some(format!("Device {device_id}: {:?}", event.event,));

            match event.event {
                EventType::Connected => {
                    if self.selected_id.is_none() {
                        self.selected_id = Some(device_id);
                    }

                    if let Some(gamepad) = self.gilrs.connected_gamepad(event.id) {
                        info!(
                            device_id,
                            name = %gamepad.name(),
                            "controller connected"
                        );
                    }
                }
                EventType::Disconnected => {
                    info!(device_id, "controller disconnected");

                    self.raw_controls.retain(|(id, _), _| *id != device_id);

                    if self.selected_id == Some(device_id) {
                        self.selected_id = None;
                    }
                }
                EventType::AxisChanged(axis, value, code) => {
                    let control = format!("{code:?}");

                    self.raw_controls.insert(
                        (device_id, control.clone()),
                        RawControlSnapshot {
                            control,
                            kind: "Axis".to_owned(),
                            logical_name: format!("{axis:?}",),
                            value,
                        },
                    );
                }
                EventType::ButtonChanged(button, value, code) => {
                    self.update_raw_analog_button(device_id, button, value, code);
                }
                EventType::ButtonPressed(button, code)
                | EventType::ButtonRepeated(button, code) => {
                    self.update_raw_digital_button(device_id, button, 1.0, code);
                }
                EventType::ButtonReleased(button, code) => {
                    self.update_raw_digital_button(device_id, button, 0.0, code);
                }
                _ => {}
            }
        }

        if self.selected_id.is_none() {
            self.selected_id = self.gilrs.gamepads().next().map(|(id, _)| usize::from(id));
        }

        self.gilrs.inc();
    }

    pub fn devices(&self) -> Vec<ControllerDevice> {
        self.gilrs
            .gamepads()
            .map(|(id, gamepad)| ControllerDevice {
                id: usize::from(id),
                name: gamepad.name().to_owned(),
                os_name: gamepad.os_name().to_owned(),
                vendor_id: gamepad.vendor_id(),
                product_id: gamepad.product_id(),
                mapping_source: format!("{:?}", gamepad.mapping_source(),),
            })
            .collect()
    }

    pub fn snapshot(&self) -> Option<ControllerSnapshot> {
        let selected_id = self.selected_id?;

        let (_, gamepad) = self
            .gilrs
            .gamepads()
            .find(|(id, _)| usize::from(*id) == selected_id)?;

        let device = ControllerDevice {
            id: selected_id,
            name: gamepad.name().to_owned(),
            os_name: gamepad.os_name().to_owned(),
            vendor_id: gamepad.vendor_id(),
            product_id: gamepad.product_id(),
            mapping_source: format!("{:?}", gamepad.mapping_source(),),
        };

        let axes = AXES
            .iter()
            .filter_map(|axis| {
                gamepad.axis_code(*axis)?;

                Some(AxisSnapshot {
                    name: format!("{axis:?}"),
                    value: gamepad.value(*axis),
                })
            })
            .collect();

        let buttons = BUTTONS
            .iter()
            .filter_map(|button| {
                let data = gamepad.button_data(*button)?;

                Some(ButtonSnapshot {
                    name: format!("{button:?}"),
                    value: data.value(),
                    pressed: data.is_pressed(),
                })
            })
            .collect();

        let raw_controls = self
            .raw_controls
            .iter()
            .filter_map(|((device_id, _), control)| {
                if *device_id == selected_id {
                    Some(control.clone())
                } else {
                    None
                }
            })
            .collect();

        Some(ControllerSnapshot {
            device,
            axes,
            buttons,
            raw_controls,
            last_event: self.last_event.clone(),
        })
    }

    fn update_raw_analog_button(
        &mut self,
        device_id: usize,
        button: Button,
        value: f32,
        code: gilrs::ev::Code,
    ) {
        let control = format!("{code:?}");

        self.raw_controls.insert(
            (device_id, control.clone()),
            RawControlSnapshot {
                control,
                kind: "Analog button".to_owned(),
                logical_name: format!("{button:?}"),
                value,
            },
        );
    }

    fn update_raw_digital_button(
        &mut self,
        device_id: usize,
        button: Button,
        value: f32,
        code: gilrs::ev::Code,
    ) {
        let control = format!("{code:?}");
        let key = (device_id, control.clone());

        if self
            .raw_controls
            .get(&key)
            .is_some_and(|snapshot| snapshot.kind == "Analog button")
        {
            return;
        }

        self.raw_controls.insert(
            key,
            RawControlSnapshot {
                control,
                kind: "Button".to_owned(),
                logical_name: format!("{button:?}"),
                value,
            },
        );
    }
}
