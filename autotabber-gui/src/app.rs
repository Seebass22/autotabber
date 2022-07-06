use eframe::egui;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

pub struct GUI {
    full: bool,
    count: u8,
    buffer_size: usize,
    min_volume: f64,
    output: String,
    receiver: Option<Receiver<String>>,
}

impl Default for GUI {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        let gui = GUI {
            full: false,
            count: 4,
            buffer_size: 512,
            min_volume: 0.6,
            output: "".to_string(),
            receiver: Some(receiver),
        };

        std::thread::spawn(move || {
            autotabber::run(
                gui.buffer_size,
                gui.count,
                gui.full,
                gui.min_volume,
                Some(sender),
            );
        });
        gui
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
            let _response = ui.add(egui::TextEdit::multiline(&mut self.output));
        });

        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(data) => {
                    self.output.push_str(&data);
                }
                Err(_err) => (),
            }
        }
        ctx.request_repaint();
    }
}
