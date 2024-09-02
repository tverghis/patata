#[derive(Debug, Default, Copy, Clone)]
pub struct Timer {
    count: u8,
}

impl Timer {
    pub fn tick(&mut self) {
        self.count = self.count.saturating_sub(1);
    }

    pub fn cur_count(self) -> u8 {
        self.count
    }

    pub fn set(&mut self, val: u8) {
        self.count = val;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tick_gt_0() {
        let mut timer = Timer { count: 10 };
        timer.tick();
        assert_eq!(9, timer.count);
    }

    #[test]
    fn tick_eq_0() {
        let mut timer = Timer { count: 0 };
        timer.tick();
        assert_eq!(0, timer.count);
    }
}
