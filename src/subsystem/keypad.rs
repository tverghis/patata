const NUM_KEYS: u8 = 16;

#[derive(Debug, Default, Clone)]
pub struct Keypad {
    keys: u16,
}

impl Keypad {
    pub fn is_key_pressed(&self, key: u8) -> bool {
        assert!(key < NUM_KEYS);

        self.keys & (1 << key) != 0
    }

    pub fn pressed_key(&self) -> Option<u8> {
        for i in 0..16 {
            if self.is_key_pressed(i) {
                return Some(i);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_key_pressed() {
        let mut keypad = Keypad::default();

        keypad.keys = 0b0000_0100_0000_0000;

        (0..16).for_each(|key| {
            assert_eq!(
                key == 10,
                keypad.is_key_pressed(key),
                "{key} should have been {}",
                key == 10
            );
        })
    }

    #[test]
    #[should_panic]
    fn is_key_pressed_panic() {
        let keypad = Keypad::default();
        keypad.is_key_pressed(20);
    }

    #[test]
    fn pressed_key() {
        let mut keypad = Keypad::default();

        keypad.keys = 0b0010_0000_0000;
        assert_eq!(Some(9), keypad.pressed_key());

        keypad.keys = 0;
        assert_eq!(None, keypad.pressed_key());
    }
}
