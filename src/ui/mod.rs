use eframe::egui::{self, Color32, RichText};

use crate::chip8::Chip8;

pub struct DebugInterface {
    rom_name: &'static str,
    chip8: Chip8,
}

impl DebugInterface {
    pub fn new(rom_name: &'static str, chip8: Chip8) -> Self {
        Self { rom_name, chip8 }
    }

    pub fn run(self) -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 240.0]),
            ..Default::default()
        };
        eframe::run_native(
            &format!("Chip8 Debugger - {}", self.rom_name),
            options,
            Box::new(|_cc| Ok(Box::new(self))),
        )
    }
}

impl eframe::App for DebugInterface {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.monospace("Memory".to_uppercase());
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, chunk) in self.chip8.memory.chunks(16).enumerate() {
                    let bytes_string = chunk
                        .iter()
                        .map(|byte| format!("{:02x}", byte))
                        .collect::<Vec<String>>()
                        .join(" ");
                    ui.horizontal(|ui| {
                        let mem_idx_str = format!("{:03x}", 16 * idx);
                        ui.label(
                            RichText::new(mem_idx_str)
                                .color(Color32::from_rgb(0xA0, 0xDB, 0x8E))
                                .monospace(),
                        );
                        ui.monospace(bytes_string);
                    });
                }
            });
        });
    }
}
