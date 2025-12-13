use glass_pumpkin::safe_prime;
use num_bigint::BigUint;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

// A repeatable stream (Iterator) of large prime numbers using a seed
pub struct SeededStream {
    rng: ChaCha20Rng,
    bits: usize,
}

impl SeededStream {
    pub fn new(bits: usize, seed: u64) -> Self {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
        let rng = ChaCha20Rng::from_seed(seed_bytes);
        SeededStream { rng, bits }
    }
}

// stream Iterator
impl Iterator for SeededStream {
    type Item = BigUint;

    fn next(&mut self) -> Option<Self::Item> {
        safe_prime::from_rng(self.bits, &mut self.rng).ok()
    }
}
