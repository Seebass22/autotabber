use autotabber::*;
use eframe::egui;

pub struct GUI {
    _full: bool,
    _count: u8,
    _buffer_size: usize,
    _min_volume: f64,
}

impl Default for GUI {
    fn default() -> Self {
        Self {
            _full: false,
            _count: 4,
            _buffer_size: 512,
            _min_volume: 0.6,
        }
    }
}

impl GUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for GUI {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
        });
    }
}
