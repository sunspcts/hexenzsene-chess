// Primarily used for magic bitboard initialization, maybe more in the future! But this is fine for now.

pub struct Xorshift {
    state: u64,
}

impl Default for Xorshift {
    fn default() -> Self {
        let mut rng = Xorshift { state: 0xFFFFFFFF };
        for _ in 0..100 {
            rng.next();
        }
        rng
    }
}

impl Xorshift {
    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 5;
        self.state
    }
    pub fn magic_candidate(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}
