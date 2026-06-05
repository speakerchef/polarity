#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Polarity"),
        ..Default::default()
    };
    eframe::run_native(
        "Polarity",
        native_options,
        Box::new(|cc| Ok(Box::new(polarity::PolarityApp::new(cc)))),
    )
}
