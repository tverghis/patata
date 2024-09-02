use super::{HEIGHT_PIXELS, WIDTH_PIXELS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawCoords {
    pub(super) pos_x: usize,
    pub(super) pos_y: usize,
}

impl DrawCoords {
    pub fn new(pos_x: u8, pos_y: u8) -> Self {
        Self {
            pos_x: (pos_x as usize) % WIDTH_PIXELS,
            pos_y: (pos_y as usize) % HEIGHT_PIXELS,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_opcode_within_bounds() {
        let pos_x = 1;
        let pos_y = 1;

        let coords = DrawCoords::new(pos_x, pos_y);

        assert_eq!(pos_x as usize, coords.pos_x);
        assert_eq!(pos_y as usize, coords.pos_y);
    }

    #[test]
    fn from_opcode_outside_bounds() {
        let pos_x = 65;
        let pos_y = 33;

        let coords = DrawCoords::new(pos_x, pos_y);

        assert_eq!(1, coords.pos_x); // 65 % WIDTH_PIXELS
        assert_eq!(1, coords.pos_y); // 33 % HEIGHT_PIXELS
    }
}
