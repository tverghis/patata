const WIDTH_PIXELS: usize = 64;
const HEIGHT_PIXELS: usize = 32;

#[derive(Debug, Clone)]
pub struct Video {
    inner: [u8; WIDTH_PIXELS * HEIGHT_PIXELS],
}

impl Default for Video {
    fn default() -> Self {
        Self {
            inner: [0; WIDTH_PIXELS * HEIGHT_PIXELS],
        }
    }
}

impl Video {
    pub fn clear(&mut self) {
        self.inner.fill(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clear() {
        let mut d = Video {
            inner: [99; WIDTH_PIXELS * HEIGHT_PIXELS],
        };

        d.clear();

        assert!(d.inner.iter().all(|i| *i == 0))
    }
}
