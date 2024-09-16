use anyhow::Context;
use patata::{chip8::Chip8, ui::DebugInterface};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let rom_file_name = std::env::args()
        .nth(1)
        .context("no ROM file name specified")?;
    let rom_bytes = std::fs::read(rom_file_name)?;

    let mut c = Chip8::default();
    c.load_rom(&rom_bytes)?;

    DebugInterface::new(c).run().unwrap();

    Ok(())
}
