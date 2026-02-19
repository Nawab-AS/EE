use primitive_types::U512;

use crate::small_primes::SMALL_PRIMES_U16;
pub const SMALL_PRIMES: [U512; SMALL_PRIMES_U16.len()] = {
    let mut arr = [U512::zero(); SMALL_PRIMES_U16.len()];
    let mut i = 0;
    while i < SMALL_PRIMES_U16.len() {
        arr[i] = U512([SMALL_PRIMES_U16[i] as u64, 0, 0, 0, 0, 0, 0, 0]);
        i += 1;
    }
    arr
};

fn mul_mod(a: U512, b: U512, m: U512) -> U512 {
    if m == U512::one() {
        return U512::zero();
    }
    
    let a = a % m;
    let b = b % m;
    
    if a.is_zero() || b.is_zero() {
        return U512::zero();
    }
    
    // direct multiplication
    if let Some(product) = a.checked_mul(b) {
        return product % m;
    }
    
    // Russian peasant multiplication with modular reduction
    
    let mut result = U512::zero();
    let mut multiplicand = a;
    let mut multiplier = b;
    
    while !multiplier.is_zero() {
        // Process larger chunks when possible
        if multiplier.0[0] & 1 == 1 {
            // Use checked_add for most additions, fallback only when necessary
            result = if let Some(sum) = result.checked_add(multiplicand) {
                sum % m
            } else {
                let r = result % m;
                let mc = multiplicand % m;
                if let Some(sum) = r.checked_add(mc) {
                    sum % m
                } else {
                    // This should be very rare
                    r + mc - m
                }
            };
        }
        
        multiplier = multiplier >> 1;
        if !multiplier.is_zero() {
            // Double multiplicand with optimization
            multiplicand = if let Some(doubled) = multiplicand.checked_add(multiplicand) {
                doubled % m
            } else {
                let mc = multiplicand % m;
                let doubled_mc = (mc + mc) % m;
                doubled_mc
            };
        }
    }
    
    result
}

fn mod_pow(mut base: U512, mut exp: U512, modulus: U512) -> U512 {
    if modulus == U512::one() {
        return U512::zero();
    }
    let mut result = U512::one();
    base = base % modulus;
    while !exp.is_zero() {
        if exp & U512::one() == U512::one() {
            result = mul_mod(result, base, modulus);
        }
        exp = exp >> 1;
        base = mul_mod(base, base, modulus);
    }
    result
}

fn miller_rabin(n: U512, k: u32) -> bool {
    if n < U512::from(2) {
        return false;
    }
    if n == U512::from(2) || n == U512::from(3) {
        return true;
    }
    if n % U512::from(2) == U512::zero() {
        return false;
    }
    
    let mut d = n - U512::one();
    let mut r = 0u32;
    while d % U512::from(2) == U512::zero() {
        d = d / U512::from(2);
        r += 1;
    }
    
    // Use deterministic bases for better performance
    let bases: [u64; 7] = [2, 3, 5, 7, 11, 13, 17];
    
    // Limit the number of bases tested for very large numbers
    let num_bases = std::cmp::min(k as usize, bases.len());
    
    for i in 0..num_bases {
        let a = U512::from(bases[i]);
        if a >= n {
            continue;
        }
        
        let mut x = mod_pow(a, d, n);
        if x == U512::one() || x == n - U512::one() {
            continue;
        }
        
        let mut cont = false;
        for _ in 0..r - 1 {
            x = mod_pow(x, U512::from(2), n);
            if x == n - U512::one() {
                cont = true;
                break;
            }
        }
        if !cont {
            return false;
        }
    }
    true
}

pub fn is_prime(n: U512, bits: u16) -> bool {
    if n < U512::from(2) {
        return false;
    }
    for &p in &SMALL_PRIMES {
        if n % p == U512::zero() && n != p {
            return false;
        }
    }
    
    // Use fewer rounds for large numbers
    let rounds = if bits < 32 {
        2
    } else if bits < 64 {
        3
    } else if bits < 128 {
        4
    } else if bits < 256 {
        5
    } else {
        6
    };
    miller_rabin(n, rounds)
}