const U12_MAX: u16 = 0xFFF;

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub struct IndexRegister {
    // We will only ever need 12 bits, but Rust does not have a native u12 type.
    // Storing valid values into this field will be enforced at runtime,
    // via the `IndexRegister::Load` function.
    inner: u16,
}

impl std::fmt::Debug for IndexRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl IndexRegister {
    pub fn load(&mut self, v: u16) {
        assert!(
            v <= U12_MAX,
            "tried to load too-large value {v} into IndexRegister"
        );
        self.inner = v;
    }

    pub fn get(self) -> usize {
        self.inner as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_ok() {
        let mut i = IndexRegister::default();

        i.load(0xFF);

        assert_eq!(0xFF, i.inner);
    }

    #[test]
    #[should_panic]
    fn load_bad_value() {
        let mut i = IndexRegister::default();

        i.load(U12_MAX + 1);
    }
}
