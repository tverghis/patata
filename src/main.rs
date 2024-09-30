use std::path::Path;

use anyhow::Context;
use patata::chip8::Chip8;
use patata::ui::DebugInterface;
use patata::Chip8Runner;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let rom_file = std::env::args()
        .nth(1)
        .context("no ROM file name specified")?;

    let path = std::path::PathBuf::from(&rom_file);
    let rom_bytes = std::fs::read(&path)?;

    let mut chip8 = Chip8::default();
    chip8.load_rom(&rom_bytes)?;

    let runner = Chip8Runner::new(chip8, 700)?;

    DebugInterface::new(rom_file_name(&path), runner)
        .run()
        .unwrap();

    Ok(())
}

fn rom_file_name(rom_path: &Path) -> &'static str {
    rom_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string()
        .leak()
}
