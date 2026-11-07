mod app;
mod core;
mod ui;

use app::MacroApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    core::elevate_if_needed();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 790.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("oGiuMacro"),
        ..Default::default()
    };

    eframe::run_native(
        "oGiuMacro",
        native_options,
        Box::new(|cc| Ok(Box::new(MacroApp::new(cc)))),
    )
}
