use std::path::Path;

use anyhow::Context;
use patata::{chip8::Chip8, ui::DebugInterface};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let rom_file = std::env::args()
        .nth(1)
        .context("no ROM file name specified")?;

    let path = std::path::PathBuf::from(&rom_file);
    let rom_bytes = std::fs::read(&path)?;

    let mut c = Chip8::default();
    c.load_rom(&rom_bytes)?;

    DebugInterface::new(rom_file_name(&path), c).run().unwrap();

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
