use eframe::egui;

use super::UiPage;

pub fn show(ui: &mut egui::Ui, selected_page: &mut UiPage) {
    ui.vertical(|ui| {
        ui.set_min_width(150.0);

        ui.heading("Hardware");
        ui.add_space(6.0);

        for page in UiPage::ALL {
            ui.selectable_value(selected_page, page, page.label());
        }
    });
}
