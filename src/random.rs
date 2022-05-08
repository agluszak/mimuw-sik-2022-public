#[derive(Debug, Clone, Default)]
pub struct Random {
    seed: u32,
}

impl Random {
    pub fn new(seed: u32) -> Random {
        Random { seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        let seed = self.seed;
        let seed_u64 = seed as u64;
        let value = (seed_u64 * 48271) % 2147483647;
        self.seed = value as u32;
        self.seed
    }

    pub fn range_u16(&mut self, min: u16, max: u16) -> u16 {
        let min = min as u32;
        let max = max as u32;
        let range = max - min;
        let value = self.next_u32();
        let value_in_range = (value % range) + min;
        (value_in_range % (u16::MAX as u32)) as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn seed_1234() {
        let mut random = Random::new(1234);
        assert_eq!(random.next_u32(), 59566414);
        assert_eq!(random.next_u32(), 1997250508);
        assert_eq!(random.next_u32(), 148423250);
        assert_eq!(random.next_u32(), 533254358);
        assert_eq!(random.next_u32(), 982122076);
        assert_eq!(random.next_u32(), 165739424);
    }
}
