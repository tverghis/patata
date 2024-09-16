use eframe::egui::{self, Color32, RichText};

use crate::chip8::Chip8;

const GREEN: Color32 = Color32::from_rgb(0xA0, 0xDB, 0x8E);

struct DebugInterfaceSettings {
    mem_show_zero_lines: bool,
}

impl Default for DebugInterfaceSettings {
    fn default() -> Self {
        Self {
            mem_show_zero_lines: false,
        }
    }
}

pub struct DebugInterface {
    rom_name: &'static str,
    chip8: Chip8,
    settings: DebugInterfaceSettings,
}

impl DebugInterface {
    pub fn new(rom_name: &'static str, chip8: Chip8) -> Self {
        Self {
            rom_name,
            chip8,
            settings: DebugInterfaceSettings::default(),
        }
    }

    pub fn run(self) -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 400.0]),
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
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    ui.vertical(|ui| {
                        ui.monospace("Memory".to_uppercase());
                        ui.checkbox(
                            &mut self.settings.mem_show_zero_lines,
                            RichText::new("Show zeroed lines").monospace(),
                        );
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (idx, chunk) in self.chip8.memory.chunks(16).enumerate() {
                                let is_all_zeros = chunk.iter().all(|&x| x == 0);
                                if !self.settings.mem_show_zero_lines && is_all_zeros {
                                    continue;
                                }

                                let bytes_string = chunk
                                    .iter()
                                    .map(|byte| format!("{:02x}", byte))
                                    .collect::<Vec<String>>()
                                    .join(" ");
                                ui.horizontal(|ui| {
                                    let mem_idx_str = format!("{:03x}", 16 * idx);
                                    ui.label(RichText::new(mem_idx_str).color(GREEN).monospace());
                                    let bytes_color = if !is_all_zeros {
                                        Color32::WHITE
                                    } else {
                                        Color32::DARK_GRAY
                                    };
                                    ui.label(
                                        RichText::new(bytes_string).color(bytes_color).monospace(),
                                    );
                                });
                            }
                        });
                    });
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.monospace("Registers".to_uppercase());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("PC").color(GREEN).monospace());
                            ui.label(
                                RichText::new(format!("{:03x}", self.chip8.program_counter))
                                    .color(Color32::WHITE)
                                    .monospace(),
                            );
                            ui.add_space(32.0);
                            ui.label(RichText::new("I").color(GREEN).monospace());
                            ui.label(
                                RichText::new(format!("{:03x}", self.chip8.index.get()))
                                    .color(Color32::WHITE)
                                    .monospace(),
                            );
                        });
                        ui.add_space(8.0);
                        for i in (0..16).step_by(2) {
                            let reg1_label =
                                RichText::new(format!("V{:X}", i)).color(GREEN).monospace();
                            let reg1_val =
                                RichText::new(format!("{:02x}", self.chip8.registers[i]))
                                    .color(color_for_byte(self.chip8.registers[i]))
                                    .monospace();
                            let reg2_label = RichText::new(format!("V{:X}", i + 1))
                                .color(GREEN)
                                .monospace();
                            let reg2_val =
                                RichText::new(format!("{:02x}", self.chip8.registers[i + 1]))
                                    .color(color_for_byte(self.chip8.registers[i + 1]))
                                    .monospace();
                            ui.horizontal(|ui| {
                                ui.label(reg1_label);
                                ui.label(reg1_val);
                                ui.add_space(32.0);
                                ui.label(reg2_label);
                                ui.label(reg2_val);
                            });
                        }
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("DT").color(GREEN).monospace());
                            ui.label(
                                RichText::new(format!(
                                    "{:02x}",
                                    self.chip8.delay_timer.cur_count()
                                ))
                                .color(color_for_byte(self.chip8.delay_timer.cur_count()))
                                .monospace(),
                            );
                            ui.add_space(32.0);
                            ui.label(RichText::new("ST").color(GREEN).monospace());
                            ui.label(
                                RichText::new(format!(
                                    "{:02x}",
                                    self.chip8.sound_timer.cur_count()
                                ))
                                .color(color_for_byte(self.chip8.sound_timer.cur_count()))
                                .monospace(),
                            );
                        });
                    });
                },
            );
        });
    }
}

fn color_for_byte(byte: u8) -> Color32 {
    if byte == 0 {
        Color32::DARK_GRAY
    } else {
        Color32::WHITE
    }
}
