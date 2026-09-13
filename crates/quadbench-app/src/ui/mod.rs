pub mod betaflight;
pub mod dashboard;
pub mod fault_injection;
pub mod motors;
pub mod navigation;
pub mod receiver;
pub mod shell;
pub mod status_bar;
pub mod top_bar;

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiPage {
    Dashboard,
    Receiver,
    Motors,
    Battery,
    Gps,
    Sensors,
    FaultInjection,
    Betaflight,
    Logs,
    Settings,
}

impl UiPage {
    pub const ALL: [Self; 10] = [
        Self::Dashboard,
        Self::Receiver,
        Self::Motors,
        Self::Battery,
        Self::Gps,
        Self::Sensors,
        Self::FaultInjection,
        Self::Betaflight,
        Self::Logs,
        Self::Settings,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Receiver => "Receiver",
            Self::Motors => "Motors / ESC",
            Self::Battery => "Battery",
            Self::Gps => "GPS",
            Self::Sensors => "Sensors",
            Self::FaultInjection => "Fault Injection",
            Self::Betaflight => "Betaflight",
            Self::Logs => "Logs",
            Self::Settings => "Settings",
        }
    }
}

pub fn show_placeholder(ui: &mut egui::Ui, page: UiPage) {
    ui.heading(page.label());
    ui.add_space(8.0);
    ui.label("This module has not been implemented yet.");
}
