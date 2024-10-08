#![allow(clippy::cast_lossless)]

use log::{info, trace};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    fonts::FONT_SET,
    opcode::OpCode,
    subsystem::{
        keypad::Keypad,
        reg::IndexRegister,
        timer::Timer,
        video::{DrawCoords, Video},
    },
};

const MEMORY_SIZE_BYTES: usize = 4096;
const PROG_CTR_START_ADDR: u16 = 0x200;
const MAX_ROM_SIZE_BYTES: usize = MEMORY_SIZE_BYTES - 0x200;
const FONTSET_START_ADDR: usize = 0x50;

#[derive(Debug, Clone)]
pub struct Chip8 {
    pub registers: [u8; 16],
    pub memory: [u8; MEMORY_SIZE_BYTES],
    pub index: IndexRegister,
    // `program_counter` needs to hold the maximum possible address in `memory`
    pub program_counter: u16,
    pub stack: [u16; 16],
    pub stack_pointer: u8,
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    keypad: Keypad,
    display: Video,
    rng: SmallRng,
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut memory = [0; 4096];

        memory[FONTSET_START_ADDR..(FONTSET_START_ADDR + FONT_SET.len())]
            .copy_from_slice(&FONT_SET);

        Self {
            registers: [0; 16],
            memory,
            index: IndexRegister::default(),
            program_counter: PROG_CTR_START_ADDR,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: Timer::default(),
            sound_timer: Timer::default(),
            keypad: Keypad::default(),
            display: Video::default(),
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Chip8 {
    // Reference: https://austinmorlan.com/posts/chip8_emulator/
    pub fn tick(&mut self) {
        let opcode = self.next_opcode();

        match opcode.nibbles() {
            (0x00, 0x00, 0x0E, 0x00) => self.op_00E0(),
            (0x00, 0x00, 0x0E, 0x0E) => self.op_00EE(),
            (0x01, _, _, _) => self.op_1nnn(opcode),
            (0x02, _, _, _) => self.op_2nnn(opcode),
            (0x03, _, _, _) => self.op_3xkk(opcode),
            (0x04, _, _, _) => self.op_4xkk(opcode),
            (0x05, _, _, 0x00) => self.op_5xy0(opcode),
            (0x06, _, _, _) => self.op_6xkk(opcode),
            (0x07, _, _, _) => self.op_7xkk(opcode),
            (0x08, _, _, 0x00) => self.op_8xy0(opcode),
            (0x08, _, _, 0x01) => self.op_8xy1(opcode),
            (0x08, _, _, 0x02) => self.op_8xy2(opcode),
            (0x08, _, _, 0x03) => self.op_8xy3(opcode),
            (0x08, _, _, 0x04) => self.op_8xy4(opcode),
            (0x08, _, _, 0x05) => self.op_8xy5(opcode),
            (0x08, _, _, 0x06) => self.op_8xy6(opcode),
            (0x08, _, _, 0x07) => self.op_8xy7(opcode),
            (0x08, _, _, 0x0E) => self.op_8xyE(opcode),
            (0x09, _, _, 0x00) => self.op_9xy0(opcode),
            (0x0A, _, _, _) => self.op_Annn(opcode),
            (0x0B, _, _, _) => self.op_Bnnn(opcode),
            (0x0C, _, _, _) => {
                let byte = self.rng.gen();
                self.op_Cxkk(opcode, byte);
            }
            (0x0D, _, _, _) => self.op_Dxyn(opcode),
            (0x0E, _, 0x09, 0x0E) => self.op_Ex9E(opcode),
            (0x0E, _, 0x0A, 0x01) => self.op_ExA1(opcode),
            (0x0F, _, 0x00, 0x07) => self.op_Fx07(opcode),
            (0x0F, _, 0x00, 0x0A) => self.op_Fx0A(opcode),
            (0x0F, _, 0x01, 0x05) => self.op_Fx15(opcode),
            (0x0F, _, 0x01, 0x08) => self.op_Fx18(opcode),
            (0x0F, _, 0x01, 0x0E) => self.op_Fx1E(opcode),
            (0x0F, _, 0x02, 0x09) => self.op_Fx29(opcode),
            (0x0F, _, 0x03, 0x03) => self.op_Fx33(opcode),
            (0x0F, _, 0x01, 0x55) => self.op_Fx55(opcode),
            (0x0F, _, 0x01, 0x65) => self.op_Fx65(opcode),
            _ => unreachable!("{:?}", opcode),
        }

        self.delay_timer.tick();
        self.sound_timer.tick();
    }

    pub fn load_rom(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
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

    /// CLS
    #[allow(non_snake_case)]
    fn op_00E0(&mut self) {
        trace!("CLS");
        self.display.clear();
    }

    /// RET
    #[allow(non_snake_case)]
    fn op_00EE(&mut self) {
        // TODO: ensure SP and PC are valid values
        trace!("RET");
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    /// JP addr
    fn op_1nnn(&mut self, opcode: OpCode) {
        trace!("JP addr {:?}", opcode);
        self.program_counter = opcode.nnn();
    }

    /// CALL addr
    fn op_2nnn(&mut self, opcode: OpCode) {
        trace!("CALL addr {:?}", opcode);
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = opcode.nnn();
    }

    /// SE Vx, byte
    fn op_3xkk(&mut self, opcode: OpCode) {
        trace!("SE Vx, byte {:?}", opcode);
        if self.registers[opcode.x() as usize] == opcode.kk() {
            self.program_counter += 2;
        }
    }

    /// SNE Vx, byte
    fn op_4xkk(&mut self, opcode: OpCode) {
        trace!("SNE Vx, byte {:?}", opcode);
        if self.registers[opcode.x() as usize] != opcode.kk() {
            self.program_counter += 2;
        }
    }

    /// SE Vx, Vy
    fn op_5xy0(&mut self, opcode: OpCode) {
        trace!("SE Vx, Vy {:?}", opcode);
        if self.registers[opcode.x() as usize] == self.registers[opcode.y() as usize] {
            self.program_counter += 2;
        }
    }

    /// LD Vx, byte
    fn op_6xkk(&mut self, opcode: OpCode) {
        trace!("LD Vx, byte {:?}", opcode);
        self.registers[opcode.x() as usize] = opcode.kk();
    }

    /// ADD Vx, byte
    fn op_7xkk(&mut self, opcode: OpCode) {
        trace!("ADD Vx, byte {:?}", opcode);
        self.registers[opcode.x() as usize] += opcode.kk();
    }

    /// LD Vx, Vy
    fn op_8xy0(&mut self, opcode: OpCode) {
        trace!("LD Vx, Vy {:?}", opcode);
        self.registers[opcode.x() as usize] = self.registers[opcode.y() as usize];
    }

    /// OR Vx, Vy
    fn op_8xy1(&mut self, opcode: OpCode) {
        trace!("OR Vx, Vy {:?}", opcode);
        self.registers[opcode.x() as usize] |= self.registers[opcode.y() as usize];
    }

    /// AND Vx, Vy
    fn op_8xy2(&mut self, opcode: OpCode) {
        trace!("AND Vx, Vy {:?}", opcode);
        self.registers[opcode.x() as usize] &= self.registers[opcode.y() as usize];
    }

    /// XOR Vx, Vy
    fn op_8xy3(&mut self, opcode: OpCode) {
        trace!("XOR Vx, Vy {:?}", opcode);
        self.registers[opcode.x() as usize] ^= self.registers[opcode.y() as usize];
    }

    #[allow(clippy::cast_possible_truncation)]
    /// ADD Vx, Vy
    fn op_8xy4(&mut self, opcode: OpCode) {
        trace!("ADD Vx, Vy {:?}", opcode);
        let vx = self.registers[opcode.x() as usize];
        let vy = self.registers[opcode.y() as usize];

        let (res, has_overflow) = vx.overflowing_add(vy);

        self.registers[0x0F] = has_overflow as u8;
        self.registers[opcode.x() as usize] = res;
    }

    /// SUB Vx, Vy
    fn op_8xy5(&mut self, opcode: OpCode) {
        trace!("SUB Vx, Vy {:?}", opcode);
        let vx = self.registers[opcode.x() as usize];
        let vy = self.registers[opcode.y() as usize];

        let (res, has_overflow) = vx.overflowing_sub(vy);

        self.registers[0x0F] = !has_overflow as u8;
        self.registers[opcode.x() as usize] = res;
    }

    /// SHR Vx
    fn op_8xy6(&mut self, opcode: OpCode) {
        trace!("SHR Vx {:?}", opcode);
        let vx = self.registers[opcode.x() as usize];
        self.registers[0x0F] = vx & 0b0000_0001;
        self.registers[opcode.x() as usize] = vx >> 1;
    }

    /// SUBN Vx, Vy
    fn op_8xy7(&mut self, opcode: OpCode) {
        trace!("SUBN Vx, Vy {:?}", opcode);
        let vx = self.registers[opcode.x() as usize];
        let vy = self.registers[opcode.y() as usize];

        let (res, has_overflow) = vy.overflowing_sub(vx);

        self.registers[0x0F] = !has_overflow as u8;
        self.registers[opcode.x() as usize] = res;
    }

    /// SHL Vx
    #[allow(non_snake_case)]
    fn op_8xyE(&mut self, opcode: OpCode) {
        trace!("SHL Vx {:?}", opcode);
        let vx = self.registers[opcode.x() as usize];
        self.registers[0x0F] = (vx >> 7) & 0b0000_0001;
        self.registers[opcode.x() as usize] = vx << 1;
    }

    /// SNE Vx,Vy
    fn op_9xy0(&mut self, opcode: OpCode) {
        trace!("SNE Vx, Vy {:?}", opcode);

        if self.registers[opcode.x() as usize] != self.registers[opcode.y() as usize] {
            self.program_counter += 2;
        }
    }

    /// LD I, addr
    #[allow(non_snake_case)]
    fn op_Annn(&mut self, opcode: OpCode) {
        trace!("LD I, Annn {:?}", opcode);
        self.index.load(opcode.nnn());
    }

    /// JP V0, addr
    #[allow(non_snake_case)]
    #[allow(clippy::cast_possible_truncation)]
    fn op_Bnnn(&mut self, opcode: OpCode) {
        trace!("JP V0, addr {:?}", opcode);
        self.program_counter = (self.registers[0] + (opcode.nnn() as u8)) as u16;
    }

    /// RND Vx, byte, rand
    #[allow(non_snake_case)]
    fn op_Cxkk(&mut self, opcode: OpCode, rand_byte: u8) {
        trace!("RND Vx, byte, rand {:?} {:X}", opcode, rand_byte);
        let x = opcode.x() as usize;

        self.registers[x] = rand_byte & opcode.kk();
    }

    /// DRW Vx, Vy, nibble
    #[allow(non_snake_case)]
    fn op_Dxyn(&mut self, opcode: OpCode) {
        trace!("DRW Vx, Vy, nibble {:?}", opcode);
        let pos_x = opcode.x();
        let pos_y = opcode.y();
        let coords = DrawCoords::new(pos_x, pos_y);

        let height = opcode.n() as usize;
        self.display.draw(
            &self.memory[self.index.get()..(self.index.get() + height)],
            &coords,
        );
    }

    /// SKP Vx
    #[allow(non_snake_case)]
    fn op_Ex9E(&mut self, opcode: OpCode) {
        trace!("SKP Vx {:?}", opcode);
        let x = opcode.x() as usize;
        let key = self.registers[x];

        if self.keypad.is_key_pressed(key) {
            self.program_counter += 2;
        }
    }

    /// SKNP Vx
    #[allow(non_snake_case)]
    fn op_ExA1(&mut self, opcode: OpCode) {
        trace!("SKNP Vx {:?}", opcode);
        let x = opcode.x() as usize;
        let key = self.registers[x];

        if !self.keypad.is_key_pressed(key) {
            self.program_counter += 2;
        }
    }

    /// LD Vx, DT
    #[allow(non_snake_case)]
    fn op_Fx07(&mut self, opcode: OpCode) {
        trace!("LD Vx, DT {:?}", opcode);
        let x = opcode.x() as usize;
        self.registers[x] = self.delay_timer.cur_count();
    }

    /// LD Vx, K
    #[allow(non_snake_case)]
    fn op_Fx0A(&mut self, opcode: OpCode) {
        trace!("LD Vx, K {:?}", opcode);
        let x = opcode.x() as usize;

        match self.keypad.pressed_key() {
            Some(k) => self.registers[x] = k,
            None => self.program_counter -= 2,
        };
    }

    /// LD DT, Vx
    #[allow(non_snake_case)]
    fn op_Fx15(&mut self, opcode: OpCode) {
        trace!("LD DT, Vx {:?}", opcode);
        let x = opcode.x() as usize;

        self.delay_timer.set(self.registers[x]);
    }

    /// LD ST, Vx
    #[allow(non_snake_case)]
    fn op_Fx18(&mut self, opcode: OpCode) {
        trace!("LD ST, Vx {:?}", opcode);
        let x = opcode.x() as usize;

        self.sound_timer.set(self.registers[x]);
    }

    /// ADD I, Vx
    #[allow(non_snake_case)]
    fn op_Fx1E(&mut self, opcode: OpCode) {
        trace!("ADD I, Vx {:?}", opcode);
        self.index += self.registers[opcode.x() as usize];
    }

    /// LD F, Vx
    #[allow(non_snake_case)]
    fn op_Fx29(&mut self, opcode: OpCode) {
        trace!("LD F, Vx {:?}", opcode);
        let digit = self.registers[opcode.x() as usize];

        self.index
            .load((FONTSET_START_ADDR + (5 * digit as usize)) as u16);
    }

    /// LD B, Vx
    #[allow(non_snake_case)]
    fn op_Fx33(&mut self, opcode: OpCode) {
        trace!("LD B, Vx {:?}", opcode);
        let mut val = self.registers[opcode.x() as usize];

        for i in (0..3).rev() {
            self.memory[self.index.get() + i] = val % 10;
            val /= 10;
        }
    }

    /// LD [I], Vx
    #[allow(non_snake_case)]
    fn op_Fx55(&mut self, opcode: OpCode) {
        trace!("LD [I], Vx {:?}", opcode);
        let x = opcode.x() as usize;

        for i in 0..=x {
            self.memory[self.index.get() + i] = self.registers[i];
        }
    }

    /// LD Vx, I
    #[allow(non_snake_case)]
    fn op_Fx65(&mut self, opcode: OpCode) {
        trace!("LD Vx, I {:?}", opcode);
        let x = opcode.x() as usize;

        for i in 0..=x {
            self.registers[i] = self.memory[self.index.get() + i];
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

        assert!(c.load_rom(&rom1).is_ok());
        assert!(c.load_rom(&rom2).is_ok());
    }

    #[test]
    fn unacceptable_rom_sizes() {
        let mut c = Chip8::default();
        let rom1 = [0u8; 0];
        let rom2 = [0u8; MAX_ROM_SIZE_BYTES + 1];

        assert!(c.load_rom(&rom1).is_err());
        assert!(c.load_rom(&rom2).is_err());
    }

    #[test]
    fn load_rom() {
        let mut c = Chip8::default();
        const ROM_LEN: usize = MAX_ROM_SIZE_BYTES - 10;
        let rom = [8u8; ROM_LEN];

        c.load_rom(&rom).unwrap();

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

        assert_eq!(
            FONT_SET,
            c.memory[FONTSET_START_ADDR..(FONTSET_START_ADDR + FONT_SET.len())]
        );
    }

    #[test]
    fn next_opcode() {
        let mut c = Chip8::default();
        c.load_rom(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();

        let opcode = c.next_opcode();
        assert_eq!(OpCode::from((0xDE, 0xAD)), opcode);
        assert_eq!(PROG_CTR_START_ADDR + 2, c.program_counter);

        let opcode = c.next_opcode();
        assert_eq!(OpCode::from((0xBE, 0xEF)), opcode);
        assert_eq!(PROG_CTR_START_ADDR + 4, c.program_counter);
    }

    #[test]
    fn jp_addr() {
        let mut c = Chip8::default();
        c.op_1nnn(OpCode::from((0x0B, 0xED)));

        assert_eq!(0xBED, c.program_counter);
    }

    #[test]
    fn call_addr() {
        let mut c = Chip8::default();
        c.program_counter = 0xFED;
        c.op_2nnn(OpCode::from((0x0B, 0xED)));

        assert_eq!(0xFED, c.stack[0]);
        assert_eq!(1, c.stack_pointer);
        assert_eq!(0xBED, c.program_counter);
    }

    #[test]
    fn skip_eq() {
        let mut c = Chip8::default();
        let old_pc = c.program_counter;

        c.registers[0] = 0xFF;

        c.op_3xkk(OpCode::from((0x00, 0xFF)));
        assert_eq!(old_pc + 2, c.program_counter);

        c.op_3xkk(OpCode::from((0x00, 0xAA)));
        assert_eq!(old_pc + 2, c.program_counter);
    }

    #[test]
    fn skip_neq() {
        let mut c = Chip8::default();
        let old_pc = c.program_counter;

        c.registers[0] = 0xFF;

        c.op_4xkk(OpCode::from((0x00, 0xFF)));
        assert_eq!(old_pc, c.program_counter);

        c.op_4xkk(OpCode::from((0x00, 0xAA)));
        assert_eq!(old_pc + 2, c.program_counter);
    }

    #[test]
    fn skip_if_reg_eq() {
        let mut c = Chip8::default();
        let old_pc = c.program_counter;

        c.registers[0] = 0xFF;
        c.registers[1] = 0xFF;
        c.registers[2] = 0xEE;

        c.op_5xy0(OpCode::from((0x00, 0x10)));
        assert_eq!(old_pc + 2, c.program_counter);

        c.op_5xy0(OpCode::from((0x00, 0x20)));
        assert_eq!(old_pc + 2, c.program_counter);
    }

    #[test]
    fn load_byte() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x6F, 0xFF));

        c.op_6xkk(opcode);
        assert_eq!(0xFF, c.registers[0x0F]);
    }

    #[test]
    fn add_byte() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x7F, 0x01));

        c.registers[0x0F] = 0x01;

        c.op_7xkk(opcode);
        assert_eq!(0x02, c.registers[0x0F]);
    }

    #[test]
    fn load_reg() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x10));

        c.registers[0x01] = 0xFF;

        c.op_8xy0(opcode);
        assert_eq!(c.registers[0x01], c.registers[0x00]);
    }

    #[test]
    fn or_reg() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x10));

        c.registers[0x00] = 0xBE;
        c.registers[0x01] = 0x22;

        c.op_8xy1(opcode);
        assert_eq!(0xBE | 0x22, c.registers[0x00]);
    }

    #[test]
    fn and_reg() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x10));

        c.registers[0x00] = 0xBE;
        c.registers[0x01] = 0x22;

        c.op_8xy2(opcode);
        assert_eq!(0xBE & 0x22, c.registers[0x00]);
    }

    #[test]
    fn xor_reg() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x10));

        c.registers[0x00] = 0xBE;
        c.registers[0x01] = 0x22;

        c.op_8xy3(opcode);
        assert_eq!(0xBE ^ 0x22, c.registers[0x00]);
    }

    #[test]
    fn add_reg_no_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x14));

        c.registers[0x00] = 0xFE;
        c.registers[0x01] = 0x01;

        c.op_8xy4(opcode);
        assert_eq!(0xFF, c.registers[0x00]);
        assert_eq!(0, c.registers[0x0F]);
    }

    #[test]
    fn add_reg_minimal_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x14));

        c.registers[0x00] = 0b1111_1111;
        c.registers[0x01] = 0b0000_0001;

        c.op_8xy4(opcode);
        assert_eq!(0x00, c.registers[0x00]);
        assert_eq!(1, c.registers[0x0F]);
    }

    #[test]
    fn add_reg_max_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x14));

        c.registers[0x00] = 0xFF;
        c.registers[0x01] = 0xFF;

        c.op_8xy4(opcode);
        assert_eq!(254, c.registers[0x00]);
        assert_eq!(1, c.registers[0x0F]);
    }

    #[test]
    fn sub_reg_no_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x15));

        c.registers[0x00] = 10;
        c.registers[0x01] = 5;

        c.op_8xy5(opcode);
        assert_eq!(5, c.registers[0x00]);
        assert_eq!(1, c.registers[0x0F]);
    }

    #[test]
    fn sub_reg_with_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x15));

        c.registers[0x00] = 5;
        c.registers[0x01] = 10;

        c.op_8xy5(opcode);
        assert_eq!(251, c.registers[0x00]);
        assert_eq!(0, c.registers[0x0F]);
    }

    #[test]
    fn shift_right() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x16));

        c.registers[0x00] = 0b0000_0110;

        c.op_8xy6(opcode);
        assert_eq!(0b0000_0011, c.registers[0x00]);
        assert_eq!(0, c.registers[0x0F]);
    }

    #[test]
    fn shift_right_with_carry() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x16));

        c.registers[0x00] = 0b0000_0111;

        c.op_8xy6(opcode);
        assert_eq!(0b0000_0011, c.registers[0x00]);
        assert_eq!(1, c.registers[0x0F]);
    }

    #[test]
    fn subn_reg_no_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x17));

        c.registers[0x00] = 5;
        c.registers[0x01] = 10;

        c.op_8xy7(opcode);
        assert_eq!(5, c.registers[0x00]);
        assert_eq!(1, c.registers[0x0F]);
    }

    #[test]
    fn subn_reg_with_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x17));

        c.registers[0x00] = 10;
        c.registers[0x01] = 5;

        c.op_8xy7(opcode);
        assert_eq!(251, c.registers[0x00]);
        assert_eq!(0, c.registers[0x0F]);
    }

    #[test]
    fn shift_left_no_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x1E));

        c.registers[0x00] = 0b0000_0110;

        c.op_8xyE(opcode);
        assert_eq!(0b0000_1100, c.registers[0x00]);
        assert_eq!(0, c.registers[0x0F]);
    }

    #[test]
    fn shift_left_with_overflow() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x80, 0x1E));

        c.registers[0x00] = 0b1100_0110;

        c.op_8xyE(opcode);
        assert_eq!(0b1000_1100, c.registers[0x00]);
        assert_eq!(1, c.registers[0x0F]);
    }

    #[test]
    fn skip_ne_reg() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0x90, 0x10));

        let old_pc = c.program_counter;

        c.registers[0x00] = 10;
        c.registers[0x01] = 10;
        c.op_9xy0(opcode);
        assert_eq!(old_pc, c.program_counter);

        c.registers[0x01] = 11;
        c.op_9xy0(opcode);
        assert_eq!(old_pc + 2, c.program_counter);
    }

    #[test]
    fn load_index() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xA1, 0x11));

        c.op_Annn(opcode);

        let mut expected_index = IndexRegister::default();
        expected_index.load(0x111);

        assert_eq!(expected_index, c.index);
    }

    #[test]
    fn jump_v0_plus_addr() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xB1, 0x11));

        c.registers[0] = 1;
        c.op_Bnnn(opcode);

        assert_eq!(0x11 + 1, c.program_counter);
    }

    #[test]
    fn random_byte_and_kk() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xC0, 0x01));

        let rand_byte = 0x11;

        c.op_Cxkk(opcode, rand_byte);

        assert_eq!(rand_byte & 0x01, c.registers[0]);
    }

    #[test]
    fn set_delay_timer() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xF0, 0x15));

        c.registers[0] = 15;

        c.op_Fx15(opcode);
        assert_eq!(15, c.delay_timer.cur_count());
    }

    #[test]
    fn set_sound_timer() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xF0, 0x18));

        c.registers[0] = 15;

        c.op_Fx18(opcode);
        assert_eq!(15, c.sound_timer.cur_count());
    }

    #[test]
    fn add_to_index() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xF0, 0x1E));

        c.index.load(10);
        c.registers[0] = 1;

        c.op_Fx1E(opcode);
        assert_eq!(11, c.index.get());
    }

    #[test]
    fn load_digit_sprite() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xF0, 0x29));

        for i in 0..=0xF {
            c.registers[0] = i;
            c.op_Fx29(opcode);

            assert_eq!(FONTSET_START_ADDR + (5 * i as usize), c.index.get());
        }
    }

    #[test]
    fn load_bcd() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xF0, 0x33));

        c.index.load(0x200);
        c.registers[0] = 234;

        c.op_Fx33(opcode);

        assert_eq!(4, c.memory[c.index.get() + 2]);
        assert_eq!(3, c.memory[c.index.get() + 1]);
        assert_eq!(2, c.memory[c.index.get()]);
    }

    #[test]
    fn store_registers() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xF3, 0x55));

        c.registers[0] = 0xDE;
        c.registers[1] = 0xAD;
        c.registers[2] = 0xBE;
        c.registers[3] = 0xEF;

        c.op_Fx55(opcode);

        let idx = c.index.get();
        assert_eq!([0xDE, 0xAD, 0xBE, 0xEF], c.memory[idx..=idx + 3]);
    }

    #[test]
    fn load_registers() {
        let mut c = Chip8::default();
        let opcode = OpCode::from((0xF3, 0x65));

        let idx = c.index.get();
        c.memory[idx + 0] = 0xDE;
        c.memory[idx + 1] = 0xAD;
        c.memory[idx + 2] = 0xBE;
        c.memory[idx + 3] = 0xEF;

        c.op_Fx65(opcode);

        assert_eq!([0xDE, 0xAD, 0xBE, 0xEF], c.registers[0..=3]);
    }
}
