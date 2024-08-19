use crate::opcode::OpCode;

const MEMORY_SIZE_BYTES: usize = 4096;
const PROG_CTR_START_ADDR: u16 = 0x200;
const MAX_ROM_SIZE_BYTES: usize = MEMORY_SIZE_BYTES - 0x200;

const FONT_SET: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug, Clone)]
pub struct Chip8 {
    pub registers: [u8; 16],
    pub memory: [u8; MEMORY_SIZE_BYTES],
    // `index` needs to hold the maximum possible address in `memory`
    pub index: u16,
    // `program_counter` needs to hold the maximum possible address in `memory`
    pub program_counter: u16,
    pub stack: [u16; 16],
    pub stack_pointer: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [u8; 16],
    pub display: [u8; 64 * 32],
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut memory = [0; 4096];

        memory[0x50..(0x50 + FONT_SET.len())].copy_from_slice(&FONT_SET);

        Self {
            registers: [0; 16],
            memory,
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

impl Chip8 {
    pub fn load_rom_from_file(&mut self, file_name: &str) -> anyhow::Result<()> {
        // TODO: don't read the file into memory if it's too large
        let file_bytes = std::fs::read(file_name)?;
        self.load_rom_bytes(&file_bytes)
    }

    fn load_rom_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let nbytes = bytes.len();

        if nbytes == 0 || nbytes > MAX_ROM_SIZE_BYTES {
            anyhow::bail!(
                "rom length is invalid. Received {} bytes, expected between {} and {} bytes.",
                nbytes,
                0,
                MAX_ROM_SIZE_BYTES
            );
        }

        let src_copy_range =
            (PROG_CTR_START_ADDR as usize)..(PROG_CTR_START_ADDR as usize + nbytes);

        self.memory[src_copy_range].copy_from_slice(bytes);

        Ok(())
    }

    fn next_opcode(&mut self) -> OpCode {
        // Opcodes are 2 bytes long.
        // `program_counter` must always point to at least 1 less than the last memory index,
        // to allow taking 2 bytes.
        assert!(
            (self.program_counter as usize + 1) < MEMORY_SIZE_BYTES,
            "program counter too large ({})",
            self.program_counter
        );

        let opcode = OpCode::from((
            self.memory[self.program_counter as usize],
            self.memory[self.program_counter as usize + 1],
        ));

        self.program_counter += 2;

        opcode
    }

    /// 00E0: CLS
    fn clear_display(&mut self) {
        self.display.fill(0);
    }

    /// 00EE: RET
    fn ret_from_sub(&mut self) {
        // TODO: ensure SP and PC are valid values
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
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

    #[test]
    fn acceptable_rom_sizes() {
        let mut c = Chip8::default();
        let rom1 = [0u8; 1];
        let rom2 = [0u8; MAX_ROM_SIZE_BYTES];

        assert!(c.load_rom_bytes(&rom1).is_ok());
        assert!(c.load_rom_bytes(&rom2).is_ok());
    }

    #[test]
    fn unacceptable_rom_sizes() {
        let mut c = Chip8::default();
        let rom1 = [0u8; 0];
        let rom2 = [0u8; MAX_ROM_SIZE_BYTES + 1];

        assert!(c.load_rom_bytes(&rom1).is_err());
        assert!(c.load_rom_bytes(&rom2).is_err());
    }

    #[test]
    fn load_rom() {
        let mut c = Chip8::default();
        const ROM_LEN: usize = MAX_ROM_SIZE_BYTES - 10;
        let rom = [8u8; ROM_LEN];

        c.load_rom_bytes(&rom).unwrap();

        // For the length of `rom`, the bytes in `memory` should be equal to `rom`.
        assert_eq!(c.memory[0x200..(0x200 + rom.len())], rom);
        // The remaining bytes in `memory` should be all zeroes since they should not have been touched.
        assert_eq!(
            c.memory[0x200 + rom.len()..],
            [0; MAX_ROM_SIZE_BYTES - ROM_LEN]
        );
    }

    #[test]
    fn load_font_set() {
        let c = Chip8::default();

        assert_eq!(FONT_SET, c.memory[0x50..(0x50 + FONT_SET.len())]);
    }

    #[test]
    fn next_opcode() {
        let mut c = Chip8::default();
        c.load_rom_bytes(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();

        let opcode = c.next_opcode();
        assert_eq!(OpCode::from((0xDE, 0xAD)), opcode);
        assert_eq!(PROG_CTR_START_ADDR + 2, c.program_counter);

        let opcode = c.next_opcode();
        assert_eq!(OpCode::from((0xBE, 0xEF)), opcode);
        assert_eq!(PROG_CTR_START_ADDR + 4, c.program_counter);
    }

    #[test]
    fn cls() {
        let mut c = Chip8::default();
        c.display.fill(99);

        c.clear_display();

        assert!(c.display.iter().all(|&i| i == 0));
    }
}
