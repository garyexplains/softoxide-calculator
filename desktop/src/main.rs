#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
use eframe::egui;
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SoftOxide Calculator")
            .with_inner_size([440.0, 600.0])
            .with_min_inner_size([400.0, 600.0])
            .with_app_id("com.softoxide.calculator"),
        ..Default::default()
    };
    eframe::run_native(
        "SoftOxide Calculator",
        options,
        Box::new(|cc| Ok(Box::new(app::Desktop::new(cc)))),
    )
}
