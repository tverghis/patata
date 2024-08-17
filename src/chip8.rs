const PROG_CTR_START_ADDR: u16 = 0x200;

#[derive(Debug, Clone)]
pub struct Chip8 {
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    // `index` needs to hold the maximum possible address in `memory`
    pub index: u16,
    // `program_counter` needs to hold the maximum possible address in `memory`
    pub program_counter: u16,
    // `stack` needs to hold up to 16 different `program_counter`s
    pub stack: [u16; 16],
    pub stack_pointer: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [u8; 16],
    pub display: [u8; 64 * 32],
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            program_counter: PROG_CTR_START_ADDR,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            display: [0; 64 * 32],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pc_starts_at_correct_address() {
        let c = Chip8::default();

        assert_eq!(c.program_counter, PROG_CTR_START_ADDR);
    }
}
