use eframe::egui;
use quadbench_betaflight::{ConfiguratorProxySnapshot, SitlSnapshot};
use quadbench_core::state::QuadState;
use quadbench_input::{ControllerDevice, ControllerSnapshot, PocketSnapshot};

use super::UiPage;

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut egui::Ui,
    state: &mut QuadState,
    selected_page: &mut UiPage,
    controller_devices: &[ControllerDevice],
    controller_snapshot: Option<&ControllerSnapshot>,
    pocket_snapshot: Option<&PocketSnapshot>,
    controller_error: Option<&str>,
    sitl_snapshot: Option<&SitlSnapshot>,
    sitl_error: Option<&str>,
    configurator_proxy_snapshot: Option<&ConfiguratorProxySnapshot>,
    configurator_proxy_error: Option<&str>,
) {
    super::top_bar::show(ui, state);

    ui.separator();

    let status_reserve = 30.0;

    let content_height = (ui.available_height() - status_reserve).max(120.0);

    let content_width = ui.available_width();

    ui.allocate_ui_with_layout(
        egui::vec2(content_width, content_height),
        egui::Layout::left_to_right(egui::Align::TOP),
        |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(160.0, content_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    super::navigation::show(ui, selected_page);
                },
            );

            ui.separator();

            let page_size = ui.available_size();

            ui.allocate_ui_with_layout(
                page_size,
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::both()
                        .id_salt("main_content_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(700.0);

                            show_page(
                                ui,
                                state,
                                *selected_page,
                                controller_devices,
                                controller_snapshot,
                                pocket_snapshot,
                                controller_error,
                                sitl_snapshot,
                                sitl_error,
                                configurator_proxy_snapshot,
                                configurator_proxy_error,
                            );
                        });
                },
            );
        },
    );

    ui.separator();

    super::status_bar::show(ui, state);
}

#[allow(clippy::too_many_arguments)]
fn show_page(
    ui: &mut egui::Ui,
    state: &mut QuadState,
    selected_page: UiPage,
    controller_devices: &[ControllerDevice],
    controller_snapshot: Option<&ControllerSnapshot>,
    pocket_snapshot: Option<&PocketSnapshot>,
    controller_error: Option<&str>,
    sitl_snapshot: Option<&SitlSnapshot>,
    sitl_error: Option<&str>,
    configurator_proxy_snapshot: Option<&ConfiguratorProxySnapshot>,
    configurator_proxy_error: Option<&str>,
) {
    match selected_page {
        UiPage::Dashboard => {
            super::dashboard::show(ui, state);
        }
        UiPage::Receiver => {
            super::receiver::show(
                ui,
                state,
                controller_devices,
                controller_snapshot,
                pocket_snapshot,
                controller_error,
            );
        }
        UiPage::Motors => {
            super::motors::show(ui, state);
        }
        UiPage::FaultInjection => {
            super::fault_injection::show(ui, state);
        }
        UiPage::Betaflight => {
            super::betaflight::show(
                ui,
                state,
                sitl_snapshot,
                sitl_error,
                configurator_proxy_snapshot,
                configurator_proxy_error,
            );
        }
        page => {
            super::show_placeholder(ui, page);
        }
    }
}
