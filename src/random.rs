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
        let value = (seed_u64 * 279410273) % 4294967291;
        self.seed = value as u32;
        seed
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
    fn seed_1() {
        let mut random = Random::new(1);
        assert_eq!(random.next_u32(), 1);
        assert_eq!(random.next_u32(), 279410273);
        assert_eq!(random.next_u32(), 3468058228);
        assert_eq!(random.next_u32(), 2207013437);
        assert_eq!(random.next_u32(), 1650159168);
    }

    #[test]
    fn seed_200000000() {
        let mut random = Random::new(200000000);
        assert_eq!(random.next_u32(), 200000000);
        assert_eq!(random.next_u32(), 3248565286);
        assert_eq!(random.next_u32(), 338750614);
        assert_eq!(random.next_u32(), 4026670339);
        assert_eq!(random.next_u32(), 1429408516);
    }
}
