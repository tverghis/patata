use log::{info, trace};

use crate::{fonts::FONT_SET, opcode::OpCode};

const MEMORY_SIZE_BYTES: usize = 4096;
const PROG_CTR_START_ADDR: u16 = 0x200;
const MAX_ROM_SIZE_BYTES: usize = MEMORY_SIZE_BYTES - 0x200;

#[derive(Debug, Clone)]
pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; MEMORY_SIZE_BYTES],
    // `index` needs to hold the maximum possible address in `memory`
    index: u16,
    // `program_counter` needs to hold the maximum possible address in `memory`
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    display: [u8; 64 * 32],
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

        info!("loading rom from file {}", file_name);

        let file_bytes = std::fs::read(file_name)?;
        self.load_rom_bytes(&file_bytes)
    }

    pub fn tick(&mut self) {
        let opcode = self.next_opcode();

        match opcode.nibbles() {
            (0x00, 0x00, 0x0E, 0x00) => self.clear_display(),
            (0x00, 0x00, 0x0E, 0x0E) => self.ret_from_sub(),
            (0x01, _, _, _) => self.jump_to_addr(opcode),
            (0x02, _, _, _) => self.call_sub(opcode),
            (0x03, _, _, _) => self.skip_if_eq(opcode),
            (0x04, _, _, _) => self.skip_if_neq(opcode),
            _ => unimplemented!(),
        }
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

        info!("loaded {} bytes into memory", nbytes);

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
        trace!("CLS");
        self.display.fill(0);
    }

    /// 00EE: RET
    fn ret_from_sub(&mut self) {
        // TODO: ensure SP and PC are valid values
        trace!("RET");
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    /// 1nnn: JP addr
    fn jump_to_addr(&mut self, opcode: OpCode) {
        trace!("JP addr {:?}", opcode);
        self.program_counter = opcode.nnn();
    }

    /// 2nnn: CALL addr
    fn call_sub(&mut self, opcode: OpCode) {
        trace!("CALL addr {:?}", opcode);
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = opcode.nnn();
    }

    /// 3xkk: SE Vx, byte
    fn skip_if_eq(&mut self, opcode: OpCode) {
        trace!("SE Vx, byte {:?}", opcode);
        if self.registers[opcode.x() as usize] == opcode.kk() {
            self.program_counter += 2;
        }
    }

    /// 4xkk: SNE Vx, byte
    fn skip_if_neq(&mut self, opcode: OpCode) {
        trace!("SNE Vx, byte {:?}", opcode);
        if self.registers[opcode.x() as usize] != opcode.kk() {
            self.program_counter += 2;
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

    #[test]
    fn jp_addr() {
        let mut c = Chip8::default();
        c.jump_to_addr(OpCode::from((0x0B, 0xED)));

        assert_eq!(0xBED, c.program_counter);
    }

    #[test]
    fn call_addr() {
        let mut c = Chip8::default();
        c.program_counter = 0xFED;
        c.call_sub(OpCode::from((0x0B, 0xED)));

        assert_eq!(0xFED, c.stack[0]);
        assert_eq!(1, c.stack_pointer);
        assert_eq!(0xBED, c.program_counter);
    }

    #[test]
    fn skip_eq() {
        let mut c = Chip8::default();
        let old_pc = c.program_counter;

        c.registers[0] = 0xFF;

        c.skip_if_eq(OpCode::from((0x00, 0xFF)));
        assert_eq!(old_pc + 2, c.program_counter);

        c.skip_if_eq(OpCode::from((0x00, 0xAA)));
        assert_eq!(old_pc + 2, c.program_counter);
    }

    #[test]
    fn skip_neq() {
        let mut c = Chip8::default();
        let old_pc = c.program_counter;

        c.registers[0] = 0xFF;

        c.skip_if_neq(OpCode::from((0x00, 0xFF)));
        assert_eq!(old_pc, c.program_counter);

        c.skip_if_neq(OpCode::from((0x00, 0xAA)));
        assert_eq!(old_pc + 2, c.program_counter);
    }
}
