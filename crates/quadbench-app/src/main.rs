mod app;
mod ui;

use app::QuadBenchApp;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("quadbench=info")),
        )
        .init();

    info!("Starting QuadBench");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1_280.0, 800.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "QuadBench",
        native_options,
        Box::new(|creation_context| Ok(Box::new(QuadBenchApp::new(creation_context)))),
    )
}
