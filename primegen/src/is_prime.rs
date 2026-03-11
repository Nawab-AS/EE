use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::sync::LazyLock;

use crate::small_primes::SMALL_PRIMES_U16;

pub static SMALL_PRIMES_BIG: LazyLock<Vec<BigUint>> = LazyLock::new(|| {
    SMALL_PRIMES_U16.iter().map(|&p| BigUint::from(p)).collect()
});

// deterministic Miller-Rabin, 12 bases covers everything we need
fn miller_rabin(n: &BigUint) -> bool {
    let one = BigUint::one();
    let two = BigUint::from(2u32);
    let n_minus_1 = n - &one;

    // factor out powers of 2: n-1 = 2^r * d
    let mut d = n_minus_1.clone();
    let mut r = 0u32;
    while (&d & &one).is_zero() {
        d >>= 1usize;
        r += 1;
    }

    let bases: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

    'outer: for &a in &bases {
        let a_big = BigUint::from(a);
        if a_big >= *n {
            continue;
        }

        let mut x = a_big.modpow(&d, n);
        if x == one || x == n_minus_1 {
            continue;
        }

        for _ in 0..r.saturating_sub(1) {
            x = x.modpow(&two, n);
            if x == n_minus_1 {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

pub fn is_prime(n: &BigUint) -> bool {
    let two = BigUint::from(2u32);
    if *n < two {
        return false;
    }
    if *n == two || *n == BigUint::from(3u32) {
        return true;
    }
    if (n & &BigUint::one()).is_zero() {
        return false;
    }

    for p in SMALL_PRIMES_BIG.iter() {
        if (n % p).is_zero() {
            return n == p;
        }
    }

    miller_rabin(n)
}