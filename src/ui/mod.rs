use eframe::egui;

pub struct DebugInterface {
    memory: Vec<u8>,
}

impl DebugInterface {
    pub fn new(memory: &[u8]) -> Self {
        Self {
            memory: memory.into(),
        }
    }

    pub fn run(self) -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
            ..Default::default()
        };
        eframe::run_native("Chip8", options, Box::new(|_cc| Ok(Box::new(self))))
    }
}

impl eframe::App for DebugInterface {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Debugger");
        });
    }
}
