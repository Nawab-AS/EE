use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{One, Zero};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};

mod conversions;
mod small_primes;
mod is_prime;

use crate::conversions::ECC_V_RSA;
use crate::is_prime::{is_prime, SMALL_PRIMES_BIG};

const TRIALS: u8 = 20;
const SEED: u32 = 873267326;
const PRIME_SEARCH_LIMIT: u32 = 5_000_000;
const LCG_A: u64 = 6364136223846793005;
const LCG_C: u64 = 1442695040888963407;
const RSA_PUBLIC_EXPONENT: u32 = 65537;
const HEX_CHARS_U256: usize = 64;
const HEX_CHARS_U1024: usize = 256;
const HEX_CHARS_U2048: usize = 512;

// simple LCG-based PRNG, deterministic (no OS randomness)
fn simple_rand(seed: &mut BigUint, max: &BigUint) -> BigUint {
    if max.is_zero() || *max == BigUint::one() {
        return BigUint::zero();
    }
    let a = BigUint::from(LCG_A);
    let c = BigUint::from(LCG_C);
    *seed = (seed.clone() * &a + &c) % max;
    seed.clone()
}

fn bit_bounds(bits: u16) -> (BigUint, BigUint) {
    let min = BigUint::one() << (bits as usize - 1);
    let max = (BigUint::one() << bits as usize) - BigUint::one();
    (min, max)
}

fn is_composite_by_small_primes(value: &BigUint) -> bool {
    SMALL_PRIMES_BIG
        .iter()
        .any(|p| (value % p).is_zero() && value != p)
}

// find the next prime >= start that fits in the given bit width
fn next_prime(start: &BigUint, bits: u16) -> BigUint {
    let (min_val, max_val) = bit_bounds(bits);
    let two = BigUint::from(2u32);

    let mut candidate = start.clone() | BigUint::one();
    if candidate < min_val {
        candidate = &min_val | BigUint::one();
    }

    for _ in 0..PRIME_SEARCH_LIMIT {
        if candidate > max_val {
            candidate = &min_val | BigUint::one();
        }

        if !is_composite_by_small_primes(&candidate) && is_prime(&candidate) {
            return candidate;
        }

        candidate += &two;
    }

    panic!("Could not find prime for {} bits", bits);
}

// find the next safe prime p = 2q+1 (both p and q prime)
fn next_safe_prime(start: &BigUint, bits: u16) -> BigUint {
    let (min_val, max_val) = bit_bounds(bits);
    let q_min = &min_val >> 1usize;
    let q_max = &max_val >> 1usize;
    let two = BigUint::from(2u32);

    let mut q = (start >> 1usize) | BigUint::one();
    if q < q_min {
        q = &q_min | BigUint::one();
    }

    for _ in 0..PRIME_SEARCH_LIMIT {
        if q > q_max {
            q = &q_min | BigUint::one();
        }

        if !is_composite_by_small_primes(&q) {
            let p_candidate = (&q << 1usize) + BigUint::one();

            if !is_composite_by_small_primes(&p_candidate)
                && p_candidate >= min_val
                && p_candidate <= max_val
            {
                if is_prime(&q) && is_prime(&p_candidate) {
                    return p_candidate;
                }
            }
        }
        q += &two;
    }
    panic!("Could not find safe prime for {} bits", bits);
}

struct TrialResult {
    ecc_bits: u16,
    rsa_bits: u16,
    session_key: BigUint,
    exponent: BigUint,
    p: BigUint,
    q: BigUint,
    ecc_prime: BigUint,
    gen_x: BigUint,
    gen_y: BigUint,
    ecc_private_key1: BigUint,
    ecc_private_key2: BigUint,
}

fn generate_trials(ecc_bits: u16, rsa_bits: u16) -> Vec<TrialResult> {
    let rsa_prime_bits = rsa_bits / 2;
    let seed_base = BigUint::from(SEED);
    let mut seed2 = &seed_base + BigUint::from(ecc_bits as u64);

    // Deterministic starting candidate for RSA primes
    let (rsa_min, rsa_max) = bit_bounds(rsa_prime_bits);
    let rsa_range = &rsa_max - &rsa_min + BigUint::one();
    let seed_mult = &seed_base * BigUint::from(rsa_bits as u64) * BigUint::from(LCG_A);
    let seed_add = &seed_mult + BigUint::from(LCG_C);
    let mut rsa_candidate = &rsa_min + (&seed_add % &rsa_range);
    rsa_candidate = &rsa_candidate | &rsa_min;

    // Deterministic starting candidate for ECC primes
    let (ecc_min, ecc_max) = bit_bounds(ecc_bits);
    let ecc_range = &ecc_max - &ecc_min + BigUint::one();
    let seed_mult_ecc = &seed_base * BigUint::from(ecc_bits as u64) * BigUint::from(LCG_A);
    let seed_add_ecc = &seed_mult_ecc + BigUint::from(LCG_C);
    let mut ecc_candidate = &ecc_min + (&seed_add_ecc % &ecc_range);
    ecc_candidate = &ecc_candidate | &ecc_min;

    let mut results = Vec::with_capacity(TRIALS as usize);

    for _ in 0..TRIALS {
        // RSA: two regular primes of rsa_prime_bits each
        let p1 = next_prime(&rsa_candidate, rsa_prime_bits);
        rsa_candidate = &p1 + BigUint::from(4u32);
        let p2 = next_prime(&rsa_candidate, rsa_prime_bits);
        rsa_candidate = &p2 + BigUint::from(4u32);

        // ECC: one safe prime of ecc_bits
        let p3 = next_safe_prime(&ecc_candidate, ecc_bits);
        ecc_candidate = &p3 + BigUint::from(4u32);

        // RSA: make sure e is coprime to totient
        let modulus = &p1 * &p2;
        let totient = (&p1 - BigUint::one()) * (&p2 - BigUint::one());
        let exponent = BigUint::from(RSA_PUBLIC_EXPONENT);
        assert!(
            totient.gcd(&exponent) == BigUint::one(),
            "e={} not coprime to totient for {}-bit RSA primes",
            RSA_PUBLIC_EXPONENT,
            rsa_prime_bits
        );

        let max_session = if modulus > BigUint::from(3u32) {
            &modulus - BigUint::from(3u32)
        } else {
            BigUint::from(100u32)
        };
        let session_key = simple_rand(&mut seed2, &max_session) + BigUint::from(2u32);

        // ECC parameters
        let p3_minus_1 = &p3 - BigUint::one();
        let gen_x = if p3 > BigUint::one() {
            simple_rand(&mut seed2, &p3_minus_1) + BigUint::one()
        } else {
            BigUint::from(2u32)
        };
        let gen_y = if p3 > BigUint::one() {
            simple_rand(&mut seed2, &p3_minus_1) + BigUint::one()
        } else {
            BigUint::from(2u32)
        };
        let ecc_private_key1 = if p3 > BigUint::one() {
            simple_rand(&mut seed2, &p3_minus_1) + BigUint::one()
        } else {
            BigUint::from(2u32)
        };
        let ecc_private_key2 = if p3 > BigUint::one() {
            simple_rand(&mut seed2, &p3_minus_1) + BigUint::one()
        } else {
            BigUint::from(3u32)
        };

        results.push(TrialResult {
            ecc_bits,
            rsa_bits,
            session_key,
            exponent,
            p: p1,
            q: p2,
            ecc_prime: p3,
            gen_x,
            gen_y,
            ecc_private_key1,
            ecc_private_key2,
        });
    }
    results
}

fn biguint_to_be_hex(v: &BigUint, num_hex_chars: usize) -> String {
    let hex = format!("{:x}", v);
    assert!(
        hex.len() <= num_hex_chars,
        "Value too large for {}-char hex ({} chars needed): {:x}",
        num_hex_chars,
        hex.len(),
        v
    );
    format!("{:0>width$}", hex, width = num_hex_chars)
}

fn fmt_u256(v: &BigUint) -> String {
    format!("U256::from_be_hex(\"{}\")", biguint_to_be_hex(v, HEX_CHARS_U256))
}

fn fmt_u1024(v: &BigUint) -> String {
    format!("U1024::from_be_hex(\"{}\")", biguint_to_be_hex(v, HEX_CHARS_U1024))
}

fn fmt_u2048(v: &BigUint) -> String {
    format!("U2048::from_be_hex(\"{}\")", biguint_to_be_hex(v, HEX_CHARS_U2048))
}

fn main() {
    let _ = &*SMALL_PRIMES_BIG; // force init

    let all_results: Vec<Vec<TrialResult>> = ECC_V_RSA
        .par_iter()
        .map(|&(ecc_bits, rsa_bits)| {
            let results = generate_trials(ecc_bits, rsa_bits);
            eprintln!(
                "ECC {} bits / RSA {} bits - {} trials generated",
                ecc_bits, rsa_bits, results.len()
            );
            results
        })
        .collect();

    let entries: Vec<_> = all_results.iter().flat_map(|v| v.iter()).collect();

    let file = File::create("./lookup.rs").expect("Failed to create lookup.rs");
    let mut w = BufWriter::new(file);

    writeln!(w, "// This file is auto-generated. Do not edit manually.").unwrap();
    writeln!(w, "use crypto_bigint::{{U256, U1024, U2048}};\n").unwrap();

    writeln!(w, "// curve: y^2 = x^3 + ax + b (mod p)").unwrap();
    writeln!(w, "pub const CURVE_A: U256 = {};", fmt_u256(&BigUint::from(2u32))).unwrap();
    writeln!(w, "pub const CURVE_B: U256 = {};", fmt_u256(&BigUint::from(3u32))).unwrap();
    writeln!(w, "pub const TRIALS: u8 = {};\n", TRIALS).unwrap();

    // Conversion table
    writeln!(w, "pub const ECC_V_RSA: [(u16, u16); {}] = [", ECC_V_RSA.len()).unwrap();
    for &(ecc, rsa) in &ECC_V_RSA {
        writeln!(w, "    ({}, {}),", ecc, rsa).unwrap();
    }
    writeln!(w, "];\n").unwrap();

    // Struct definitions
    writeln!(w, "#[derive(Clone, Copy)]").unwrap();
    writeln!(w, "pub struct RSA {{ pub session_key: U2048, pub exponent: U256, pub p: U1024, pub q: U1024 }}\n").unwrap();
    writeln!(w, "#[derive(Clone, Copy)]").unwrap();
    writeln!(w, "pub struct Point {{ pub x: U256, pub y: U256 }}\n").unwrap();
    writeln!(w, "#[derive(Clone, Copy)]").unwrap();
    writeln!(w, "pub struct EccCurve {{ pub a: U256, pub b: U256, pub p: U256, pub generator: Point }}\n").unwrap();
    writeln!(w, "#[derive(Clone, Copy)]").unwrap();
    writeln!(w, "pub struct ECC {{ pub curve: EccCurve, pub private_key1: U256, pub private_key2: U256 }}\n").unwrap();
    writeln!(w, "#[derive(Clone, Copy)]").unwrap();
    writeln!(w, "pub struct KeySize {{ pub ecc_bits: u16, pub rsa_bits: u16, pub rsa: RSA, pub ecc: ECC }}\n").unwrap();

    // Lookup array
    writeln!(w, "pub const LOOKUP_TABLE: [KeySize; {}] = [", entries.len()).unwrap();
    for (i, e) in entries.iter().enumerate() {
        let comma = if i < entries.len() - 1 { "," } else { "" };
        writeln!(w, "    KeySize {{").unwrap();
        writeln!(w, "        ecc_bits: {},", e.ecc_bits).unwrap();
        writeln!(w, "        rsa_bits: {},", e.rsa_bits).unwrap();
        writeln!(w, "        rsa: RSA {{").unwrap();
        writeln!(w, "            session_key: {},", fmt_u2048(&e.session_key)).unwrap();
        writeln!(w, "            exponent: {},", fmt_u256(&e.exponent)).unwrap();
        writeln!(w, "            p: {},", fmt_u1024(&e.p)).unwrap();
        writeln!(w, "            q: {}", fmt_u1024(&e.q)).unwrap();
        writeln!(w, "        }},").unwrap();
        writeln!(w, "        ecc: ECC {{").unwrap();
        writeln!(w, "            curve: EccCurve {{").unwrap();
        writeln!(w, "                a: CURVE_A, b: CURVE_B,").unwrap();
        writeln!(w, "                p: {},", fmt_u256(&e.ecc_prime)).unwrap();
        writeln!(w, "                generator: Point {{ x: {}, y: {} }}", fmt_u256(&e.gen_x), fmt_u256(&e.gen_y)).unwrap();
        writeln!(w, "            }},").unwrap();
        writeln!(w, "            private_key1: {},", fmt_u256(&e.ecc_private_key1)).unwrap();
        writeln!(w, "            private_key2: {}", fmt_u256(&e.ecc_private_key2)).unwrap();
        writeln!(w, "        }}").unwrap();
        writeln!(w, "    }}{}", comma).unwrap();
    }
    writeln!(w, "];").unwrap();

    w.flush().unwrap();
    println!(
        "Generated lookup.rs with {} KeySize entries ({} key-size pairs x {} trials)",
        entries.len(),
        ECC_V_RSA.len(),
        TRIALS
    );
}
