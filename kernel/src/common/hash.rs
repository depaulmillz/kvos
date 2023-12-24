use core::hash::Hasher;

pub struct Mix13Hash;

impl Mix13Hash {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_hash(&self, bytes: &[u8]) -> u64 {
        let mut hash = 0u64;
        for &byte in bytes {
            hash ^= (byte as u64) << ((hash % 8) * 8);
            hash = mix13(hash);
        }
        hash
    }
}

impl Hasher for Mix13Hash {
    fn finish(&self) -> u64 {
        0 // Since it's stateless, finish doesn't have a pre-computed state to return.
    }

    fn write(&mut self, _bytes: &[u8]) {
        // No-op since it's stateless.
    }
}

// mix13 hash function
pub fn mix13(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    x
}
