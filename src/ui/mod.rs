use eframe::egui;

use crate::chip8::Chip8;

pub struct DebugInterface {
    chip8: Chip8,
}

impl DebugInterface {
    pub fn new(chip8: Chip8) -> Self {
        Self { chip8 }
    }

    pub fn run(self) -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 240.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Chip8 Debugger",
            options,
            Box::new(|_cc| Ok(Box::new(self))),
        )
    }
}

impl eframe::App for DebugInterface {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Memory");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for chunk in self.chip8.memory.chunks(16) {
                    let bytes_string = chunk
                        .iter()
                        .map(|byte| format!("{:02x}", byte))
                        .collect::<Vec<String>>()
                        .join(" ");
                    ui.monospace(bytes_string);
                }
            });
        });
    }
}
