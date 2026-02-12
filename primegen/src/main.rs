use primitive_types::U512;
use std::fs::File;
use std::io::{BufWriter, Write};

mod small_primes;
use crate::small_primes::SMALL_PRIMES_U16;
const SMALL_PRIMES: [U512; SMALL_PRIMES_U16.len()] = {
    let mut arr = [U512::zero(); SMALL_PRIMES_U16.len()];
    let mut i = 0;
    while i < SMALL_PRIMES_U16.len() {
        arr[i] = U512([SMALL_PRIMES_U16[i] as u64, 0, 0, 0, 0, 0, 0, 0]);
        i += 1;
    }
    arr
};

const PRIME_RANGE: [u32; 2] = [8, 511];
const SEED: U512 = U512([460851, 0, 0, 0, 0, 0, 0, 0]);

fn mul_mod(a: U512, b: U512, m: U512) -> U512 {
    if m == U512::one() {
        return U512::zero();
    }
    
    let a = a % m;
    let b = b % m;
    
    if a.is_zero() || b.is_zero() {
        return U512::zero();
    }
    
    // Try direct multiplication first
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

fn is_prime(n: U512, bits: u32) -> bool {
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

fn next_prime(mut n: U512, bits: u32, min: U512, max: U512) -> U512 {
    const MAX_ITER: u64 = if cfg!(debug_assertions) { 1_000_000 } else { 10_000_000 };
    if n % U512::from(2) == U512::zero() {
        n = n + U512::one();
    }
    let mut iter = 0u64;
    loop {
        if n > max {
            n = min | U512::one();
        }
        let mut divisible = false;
        for &p in &SMALL_PRIMES {
            if n % p == U512::zero() && n != p {
                divisible = true;
                break;
            }
        }
        if !divisible && is_prime(n, bits) {
            return n;
        }
        n = n + U512::from(2);
        iter += 1;
        if iter >= MAX_ITER {
            eprintln!(
                "[WARN] Timeout: Could not find a prime for {} bits after {} iterations.",
                bits, MAX_ITER
            );
            return U512::zero();
        }
    }
}

fn format_u512(value: U512) -> String {
    format!(
        "U512([{}, {}, {}, {}, {}, {}, {}, {}])",
        value.0[0], value.0[1], value.0[2], value.0[3],
        value.0[4], value.0[5], value.0[6], value.0[7]
    )
}

fn main() {
    let file = File::create("./lookup.rs").expect("Failed to create lookup.rs");
    let mut writer = BufWriter::new(file);
    
    writeln!(writer, "// This file is auto-generated. Do not edit manually.").unwrap();
    writeln!(writer, "use primitive_types::U512;").unwrap();
    writeln!(
        writer,
        "pub const PRIMES: [[U512; 5]; {}] = [",
        PRIME_RANGE[1] - PRIME_RANGE[0] + 1
    ).unwrap();
    
    for bits in PRIME_RANGE[0]..=PRIME_RANGE[1] {
        let min = U512::one() << (bits - 1);
        let max = (U512::one() << bits) - U512::one();
        
        let seed_mult = {
            let s = SEED;
            let b = U512::from(bits);
            let c = U512::from(6364136223846793005u64);
            let (temp1, _) = s.overflowing_mul(b);
            let (temp2, _) = temp1.overflowing_mul(c);
            temp2
        };
        let seed_add = seed_mult + U512::from(1442695040888963407u64);
        let range_size = max - min + U512::one();
        let mut candidate = min + (seed_add % range_size);
        candidate = candidate | min;

        let mut primes = Vec::new();
        let mut candidate_start = candidate;
        
        // Generate 5 primes
        for _ in 0..5 {
            let prime = next_prime(candidate_start, bits, min, max);
            if prime.is_zero() {
                break;
            }
            primes.push(prime);
            candidate_start = prime + U512::from(2);
        }

        // Fill remaining slots with zeros if needed
        while primes.len() < 5 {
            primes.push(U512::zero());
        }
        
        let prime_strings: Vec<String> = primes.iter().map(|p| format_u512(*p)).collect();
        let array_content = prime_strings.join(", ");
        
        if primes.iter().filter(|p| !p.is_zero()).count() < 5 {
            writeln!(writer, "    [{}], // {} bits (timeout/partial)", array_content, bits).unwrap();
            println!(
                "[WARN] Could not find 5 primes for {} bits. Found {} primes.",
                bits, primes.iter().filter(|p| !p.is_zero()).count()
            );
        } else {
            writeln!(writer, "    [{}], // {} bits", array_content, bits).unwrap();
            println!("{} bits: {}, {}, {}, {}, {}", bits, primes[0], primes[1], primes[2], primes[3], primes[4]);
        }
    }
    writeln!(writer, "];").unwrap();
    
    writer.flush().unwrap();
    println!("Generated lookup.rs successfully!");
}
