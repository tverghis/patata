use log::{trace, warn};
use std::time::{Duration, Instant};

use chip8::Chip8;

mod fonts;
mod opcode;
mod subsystem;

pub mod chip8;
pub mod platform;
pub mod ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerEvent {
    Start,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunnerState {
    NotStarted,
    Running,
    Paused,
}

#[derive(Debug)]
pub struct Chip8Runner {
    pub chip8: Chip8,
    tick_hz: usize,
    state: RunnerState,
}

impl Chip8Runner {
    pub fn new(chip8: Chip8, tick_hz: usize) -> anyhow::Result<Self> {
        Ok(Self {
            chip8,
            tick_hz,
            state: RunnerState::NotStarted,
        })
    }

    pub fn start(&mut self) {
        match self.state {
            RunnerState::Running => {
                warn!("");
            }
            RunnerState::NotStarted | RunnerState::Paused => {
                trace!("Starting runner");
                self.state = RunnerState::Running;
            }
        };

        loop {
            let now = Instant::now();

            self.chip8.tick();

            let duration = now.elapsed();
            let sleep_time = self.tick_duration_nanos() - duration;

            trace!(
                "Cycle took {}ns, sleeping for {}ms",
                duration.as_nanos(),
                sleep_time.as_millis()
            );

            std::thread::sleep(sleep_time);
        }
    }

    fn tick_duration_nanos(&self) -> Duration {
        let nanos_per_tick = 1000_000_000. / self.tick_hz as f32;
        Duration::from_nanos(nanos_per_tick as u64)
    }
}
