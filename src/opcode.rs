pub struct OpCode {
    inner: u16,
}

impl OpCode {
    pub fn nibbles(&self) -> (u8, u8, u8, u8) {
        (
            ((self.inner & 0xF000) >> 12) as u8,
            ((self.inner & 0x0F00) >> 8) as u8,
            ((self.inner & 0x00F0) >> 4) as u8,
            (self.inner & 0x000F) as u8,
        )
    }

    pub fn nnn(&self) -> u16 {
        self.inner & 0x0FFF
    }

    pub fn kk(&self) -> u8 {
        (self.inner & 0x00FF) as u8
    }

    pub fn x(&self) -> u8 {
        self.nibbles().1
    }

    pub fn y(&self) -> u8 {
        self.nibbles().2
    }

    pub fn n(&self) -> u8 {
        self.nibbles().3
    }
}

impl From<(u8, u8)> for OpCode {
    fn from(next_bytes: (u8, u8)) -> Self {
        Self {
            inner: (u16::from(next_bytes.0) << 8) | u16::from(next_bytes.1),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        let opcode = OpCode::from((0xDE, 0xAD));
        assert_eq!(0xDEAD, opcode.inner)
    }

    #[test]
    fn nibbles() {
        let opcode = OpCode::from((0xDE, 0xAD));
        assert_eq!((0x0D, 0x0E, 0x0A, 0x0D), opcode.nibbles())
    }

    #[test]
    fn nnn() {
        let opcode = OpCode::from((0xDE, 0xAD));
        assert_eq!(0x0EAD, opcode.nnn())
    }

    #[test]
    fn kk() {
        let opcode = OpCode::from((0xDE, 0xAD));
        assert_eq!(0x00AD, opcode.kk())
    }

    #[test]
    fn x() {
        let opcode = OpCode::from((0xDE, 0xAD));
        assert_eq!(0x0E, opcode.x())
    }

    #[test]
    fn y() {
        let opcode = OpCode::from((0xDE, 0xAD));
        assert_eq!(0x0A, opcode.y())
    }

    #[test]
    fn n() {
        let opcode = OpCode::from((0xBE, 0xAD));
        assert_eq!(0x0D, opcode.n())
    }
}
