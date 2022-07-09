#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
pub use app::GUI;
use eframe::egui::Vec2;

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(512.0, 384.0)),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "autotabber-gui",
        native_options,
        Box::new(|cc| Box::new(GUI::new(cc))),
    );
}
