use std::time::Duration;

use eframe::egui;
use quadbench_core::state::QuadState;

use crate::ui::{self, UiPage};

pub struct QuadBenchApp {
    state: QuadState,
    selected_page: UiPage,
}

impl QuadBenchApp {
    pub fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: QuadState::default(),
            selected_page: UiPage::Dashboard,
        }
    }
}

impl eframe::App for QuadBenchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui::top_bar::show(ui, &mut self.state);

        ui.separator();

        ui.horizontal(|ui| {
            ui::navigation::show(ui, &mut self.selected_page);

            ui.separator();

            ui.vertical(|ui| {
                ui.set_min_width(700.0);

                match self.selected_page {
                    UiPage::Dashboard => ui::dashboard::show(ui, &self.state),
                    page => ui::show_placeholder(ui, page),
                }
            });
        });

        ui.separator();

        ui::status_bar::show(ui, &self.state);

        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
}
