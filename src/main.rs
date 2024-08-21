use anyhow::Context;
use patata::chip8::Chip8;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let rom_file_name = std::env::args()
        .nth(1)
        .context("no ROM file name specified")?;

    let mut c = Chip8::default();

    c.load_rom_from_file(&rom_file_name)?;

    Ok(())
}
