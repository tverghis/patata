use anyhow::anyhow;
use sdl2::render::WindowCanvas;

pub struct VideoPlatform;

impl VideoPlatform {
    pub fn init() -> anyhow::Result<WindowCanvas> {
        let sdl_context = sdl2::init().map_err(into_anyhow)?;
        let video_subsystem = sdl_context.video().map_err(into_anyhow)?;

        let window = video_subsystem
            .window("Chip8", 800, 600)
            .position_centered()
            .build()?;

        Ok(window.into_canvas().build()?)
    }
}

fn into_anyhow(err: String) -> anyhow::Error {
    anyhow!(err)
}
