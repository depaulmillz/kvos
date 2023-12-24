use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;


pub struct Random {
    rng:  XorShiftRng,
    seed: [u8; 16],
}

impl Random {
    pub fn new() -> Self {
        let mut seed = [0u8; 16];
        for i in 0..16 {
            seed[i] = i as u8;
        }
        let rng = XorShiftRng::from_seed(seed);
        Random {
            rng,
            seed,
        }
    }

    pub fn get_random(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min + 1;
        let bias = (u64::MAX % range + 1) % range;
    
        loop {
            let random_value = self.rng.next_u64();
            if random_value >= bias {
                return min + (random_value - bias) % range;
            }
        }
    }

    pub fn get_sead(&self) -> [u8; 16] {
        self.seed
    }

    pub fn set_seed(&mut self, seed: [u8; 16]) {
        self.seed = seed;
        self.rng = XorShiftRng::from_seed(seed);
    }
}
