#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
pub use app::GUI;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "autotabber-gui",
        native_options,
        Box::new(|cc| Box::new(GUI::new(cc))),
    );
}
