use eframe::egui;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

pub struct GUI {
    full: bool,
    count: u8,
    buffer_size: usize,
    min_volume: f64,
    output: String,
    key: String,
    receiver: Option<Receiver<String>>,
}

impl Default for GUI {
    fn default() -> Self {
        Self {
            full: false,
            count: 4,
            buffer_size: 512,
            min_volume: 0.12,
            output: "".to_string(),
            key: "C".to_string(),
            receiver: None,
        }
    }
}

impl GUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // don't print error message on panic (thread exit)
        std::panic::set_hook(Box::new(|_| {}));
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
            ui.horizontal(|ui| {
                if ui.button("run").clicked() {
                    let (sender, receiver) = mpsc::channel();
                    self.receiver = Some(receiver);

                    let buffer_size = self.buffer_size;
                    let count = self.count;
                    let full = self.full;
                    let min_volume = self.min_volume;
                    let key = self.key.clone();
                    std::thread::spawn(move || {
                        autotabber::run(buffer_size, count, full, min_volume, key, Some(sender));
                    });
                }
                if ui.button("stop").clicked() {
                    self.receiver = None;
                }
            });

            egui::ComboBox::from_label("key")
                .selected_text(&self.key)
                .width(60.0)
                .show_ui(ui, |ui| {
                    for key in [
                        "C", "G", "D", "A", "E", "B", "F#", "Db", "Ab", "Eb", "Bb", "F", "LF",
                        "LC", "LD", "HG",
                    ]
                    .iter()
                    {
                        ui.selectable_value(&mut self.key, key.to_string(), *key);
                    }
                });

            ui.horizontal(|ui| {
                ui.label("buffer size:");
                let buffer_sizes: [usize; 5] = [256, 512, 1024, 2048, 4096];
                for value in buffer_sizes.iter() {
                    ui.selectable_value(&mut self.buffer_size, *value, value.to_string());
                }
            });

            ui.add(egui::Slider::new(&mut self.count, 1..=20).text("MinOccurs"));
            ui.add(egui::Slider::new(&mut self.min_volume, 0.0..=1.0).text("MinVolume"));

            ui.add(egui::TextEdit::multiline(&mut self.output));
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
