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
    measured_volume: String,
    volume_receiver: Option<Receiver<String>>,
    print_key: bool,
    about_open: bool,
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
            measured_volume: "".to_string(),
            volume_receiver: None,
            print_key: false,
            about_open: false,
        }
    }
}

impl GUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // don't print error message on panic (thread exit)
        std::panic::set_hook(Box::new(|_| {}));
        Default::default()
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .open(&mut self.about_open)
            .show(ctx, |ui| {
                ui.label("autotabber-gui");
                ui.add_space(10.0);
                ui.label("Copyright © 2022");
                ui.label("Sebastian James Thuemmel");
                ui.add_space(10.0);
                ui.add(egui::Hyperlink::from_label_and_url(
                    "source code",
                    "https://github.com/Seebass22/autotabber",
                ));
                ui.add(egui::Hyperlink::from_label_and_url(
                    "binary downloads",
                    "https://seebass22.itch.io/autotabber",
                ));
            });
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

                if ui.button("About").clicked() {
                    self.about_open = true;
                }
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

                    if self.print_key {
                        if ! self.output.is_empty() {
                            self.output.push('\n');
                        }
                        self.output.push_str(&self.key);
                        self.output.push_str(" harp:\n");
                    }

                    std::thread::spawn(move || {
                        autotabber::run(buffer_size, count, full, min_volume, key, Some(sender));
                    });
                }
                if ui.button("stop").clicked() {
                    self.receiver = None;
                }
            });

            ui.horizontal(|ui| {
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
                ui.add_space(10.0);
                ui.checkbox(&mut self.print_key, "print key");
            });

            ui.collapsing("settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("buffer size:");
                    let buffer_sizes: [usize; 5] = [256, 512, 1024, 2048, 4096];
                    for value in buffer_sizes.iter() {
                        ui.selectable_value(&mut self.buffer_size, *value, value.to_string());
                    }
                });

                ui.add(egui::Slider::new(&mut self.count, 1..=20).text("MinOccurs"));
                ui.add(egui::Slider::new(&mut self.min_volume, 0.0..=1.0).text("MinVolume"));
                ui.horizontal(|ui| {
                    if ui.button("measure volume").clicked() {
                        if self.volume_receiver.is_none() {
                            let (sender, receiver) = mpsc::channel();
                            self.volume_receiver = Some(receiver);
                            std::thread::spawn(move || autotabber::measure_volume(Some(sender)));
                        } else {
                            self.volume_receiver = None;
                        }
                    }
                    ui.label(&self.measured_volume);
                });
            });

            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.output),
            );
        });

        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(data) => {
                    self.output.push_str(&data);
                }
                Err(_err) => (),
            }
        }
        if let Some(volume_receiver) = &self.volume_receiver {
            match volume_receiver.try_recv() {
                Ok(data) => {
                    self.measured_volume = data;
                }
                Err(_err) => (),
            }
        }
        self.about_window(ctx);
        ctx.request_repaint();
    }
}
