use primitive_types::{U256, U512};
use std::fs::File;
use std::io::{BufWriter, Write};

mod small_primes;
mod is_prime;
use crate::is_prime::is_prime;
use crate::is_prime::SMALL_PRIMES;

const PRIME_RANGE: [u16; 2] = [8, 256];
const TRIALS: u8 = 5; // ECC/RSA trials per key size
const SEED: u32 = 873267326;

#[derive(Clone)]
struct RSA { // given primes p and q
    pub session_key: U512, // random number in [2, modulus-1]
    
    pub modulus: U512, // p * q
    pub totient: U512, // (p-1)(q-1)
    pub exponent: U256, // random number coprime to totient
    pub private_key: U512, // modular inverse of exponent and totient
}


#[derive(Clone)]
struct Point { 
    pub x: U256, 
    pub y: U256 
}

// Weierstrass Curve (y^2 = x^3 + ax + b mod p) where p is prime and 27a^3 + 27b^2 != 0 mod p
#[derive(Clone)]
struct EccCurve {
    pub a: U256, 
    #[allow(dead_code)]
    pub b: U256,
    pub p: U256,
    pub generator: Point,
}

#[derive(Clone)]
struct ECC {
    pub curve: EccCurve,
    
    pub private_key1: U256, // random number in [1, p-1]
    pub public_key1: Point, // private_key * base_point
    pub private_key2: U256, // random number in [1, p-1]
    pub public_key2: Point, // private_key * base_point
}

#[derive(Clone)]
struct KeySize {
    pub bits: u16,
    pub rsa: RSA,
    pub ecc: ECC,
}

fn mul_mod(a: U512, b: U512, m: U512) -> U512 {
    if m == U512::one() { return U512::zero(); }
    let a = a % m;
    let b = b % m;
    if a.is_zero() || b.is_zero() { 
        return U512::zero(); 
    }
    if let Some(product) = a.checked_mul(b) { return product % m; }
    
    // Russian peasant multiplication
    let mut result = U512::zero();
    let mut multiplicand = a;
    let mut multiplier = b;
    
    while !multiplier.is_zero() {
        if multiplier.0[0] & 1 == 1 {
            result = if let Some(sum) = result.checked_add(multiplicand) {
                sum % m
            } else {
                let r = result % m;
                let mc = multiplicand % m;
                (r + mc) % m
            };
        }
        multiplier = multiplier >> 1;
        if !multiplier.is_zero() {
            multiplicand = if let Some(doubled) = multiplicand.checked_add(multiplicand) {
                doubled % m
            } else {
                let mc = multiplicand % m;
                (mc + mc) % m
            };
        }
    }
    result
}

// Extended Euclidian GCD for modular inverse
fn extended_gcd(a: U512, b: U512) -> (U512, U512, U512, bool, bool) {
    if b.is_zero() { return (a, U512::one(), U512::zero(), false, false); }
    
    let mut old_r = a; 
    let mut r = b;
    let mut old_s = U512::one(); 
    let mut s = U512::zero();
    let mut old_t = U512::zero(); 
    let mut t = U512::one();
    let mut old_s_neg = false; let mut s_neg = false;
    let mut old_t_neg = false; let mut t_neg = true;
    
    while !r.is_zero() {
        let quotient = old_r / r;
        let temp_r = r; r = old_r - quotient * r; old_r = temp_r;
        
        let temp_s = s; let temp_s_neg = s_neg;
        let prod = quotient * s;
        if old_s_neg == s_neg {
            if old_s >= prod { s = old_s - prod; s_neg = old_s_neg; }
            else { s = prod - old_s; s_neg = !old_s_neg; }
        } else { s = old_s + prod; s_neg = old_s_neg; }
        old_s = temp_s; old_s_neg = temp_s_neg;
        
        let temp_t = t; let temp_t_neg = t_neg;
        let prod = quotient * t;
        if old_t_neg == t_neg {
            if old_t >= prod { t = old_t - prod; t_neg = old_t_neg; }
            else { t = prod - old_t; t_neg = !old_t_neg; }
        } else { t = old_t + prod; t_neg = old_t_neg; }
        old_t = temp_t; old_t_neg = temp_t_neg;
    }
    (old_r, old_s, old_t, old_s_neg, old_t_neg)
}

fn mod_inverse(a: U512, m: U512) -> Option<U512> {
    let (gcd, x, _, x_neg, _) = extended_gcd(a, m);
    if gcd != U512::one() { return None; }
    if x_neg { Some(m - (x % m)) } else { Some(x % m) }
}

fn mul_mod_u256(a: U256, b: U256, m: U256) -> U256 {
    if m == U256::one() { return U256::zero(); }
    let a = a % m; let b = b % m;
    if a.is_zero() || b.is_zero() { return U256::zero(); }
    if let Some(product) = a.checked_mul(b) { return product % m; }
    
    let mut result = U256::zero();
    let mut multiplicand = a;
    let mut multiplier = b;
    
    while !multiplier.is_zero() {
        if multiplier.0[0] & 1 == 1 {
            result = if let Some(sum) = result.checked_add(multiplicand) {
                sum % m
            } else {
                let r = result % m; let mc = multiplicand % m;
                (r + mc) % m
            };
        }
        multiplier = multiplier >> 1;
        if !multiplier.is_zero() {
            multiplicand = if let Some(doubled) = multiplicand.checked_add(multiplicand) {
                doubled % m
            } else {
                let mc = multiplicand % m;
                (mc + mc) % m
            };
        }
    }
    
    result
}

fn extended_gcd_u256(a: U256, b: U256) -> (U256, U256, U256, bool, bool) {
    if b.is_zero() { return (a, U256::one(), U256::zero(), false, false); }
    
    let mut old_r = a; let mut r = b;
    let mut old_s = U256::one(); let mut s = U256::zero();
    let mut old_t = U256::zero(); let mut t = U256::one();
    let mut old_s_neg = false; let mut s_neg = false;
    let mut old_t_neg = false; let mut t_neg = true;
    
    while !r.is_zero() {
        let quotient = old_r / r;
        let temp_r = r; r = old_r - quotient * r; old_r = temp_r;
        
        let temp_s = s; let temp_s_neg = s_neg;
        let prod = quotient * s;
        if old_s_neg == s_neg {
            if old_s >= prod { s = old_s - prod; s_neg = old_s_neg; }
            else { s = prod - old_s; s_neg = !old_s_neg; }
        } else { s = old_s + prod; s_neg = old_s_neg; }
        old_s = temp_s; old_s_neg = temp_s_neg;
        
        let temp_t = t; let temp_t_neg = t_neg;
        let prod = quotient * t;
        if old_t_neg == t_neg {
            if old_t >= prod { t = old_t - prod; t_neg = old_t_neg; }
            else { t = prod - old_t; t_neg = !old_t_neg; }
        } else { t = old_t + prod; t_neg = old_t_neg; }
        old_t = temp_t; old_t_neg = temp_t_neg;
    }
    (old_r, old_s, old_t, old_s_neg, old_t_neg)
}

fn mod_inverse_u256(a: U256, m: U256) -> Option<U256> {
    let (gcd, x, _, x_neg, _) = extended_gcd_u256(a, m);
    if gcd != U256::one() { return None; }
    if x_neg { Some(m - (x % m)) } else { Some(x % m) }
}

fn u512_to_u256(val: U512) -> U256 {
    U256([val.0[0], val.0[1], val.0[2], val.0[3]])
}

fn u256_to_u512(val: U256) -> U512 {
    U512([val.0[0], val.0[1], val.0[2], val.0[3], 0, 0, 0, 0])
}

// Deterministic pseudo-random generator using LCG (seeded, no OS randomness)
fn simple_rand(seed: &mut U512, max: U512) -> U512 {
    // LCG: X_{n+1} = (a * X_n + c) mod m
    let a = U512::from(6364136223846793005u64);
    let c = U512::from(1442695040888963407u64);
    
    if let Some(product) = seed.checked_mul(a) {
        *seed = product % max;
    } else {
        *seed = mul_mod(*seed, a, max);
    }
    
    *seed = if let Some(sum) = seed.checked_add(c) {
        sum % max
    } else { (*seed + (c % max)) % max };
    *seed
}

fn ecc_point_add(p1: &Point, p2: &Point, curve: &EccCurve) -> Point {
    // Handle point at infinity (represented as (0, 0))
    if p1.x.is_zero() && p1.y.is_zero() {
        return p2.clone();
    }
    if p2.x.is_zero() && p2.y.is_zero() {
        return p1.clone();
    }
    
    // If same x coordinate but different y, return infinity
    if p1.x == p2.x {
        if p1.y != p2.y { 
            return Point { x: U256::zero(), y: U256::zero() }; 
        }
        let numerator = mul_mod_u256(U256::from(3), mul_mod_u256(p1.x, p1.x, curve.p), curve.p);
        let numerator = (numerator + curve.a) % curve.p;
        let denominator = mul_mod_u256(U256::from(2), p1.y, curve.p);
        
        if let Some(denom_inv) = mod_inverse_u256(denominator, curve.p) {
            let slope = mul_mod_u256(numerator, denom_inv, curve.p);
            let x3 = {
                let slope_sq = mul_mod_u256(slope, slope, curve.p);
                let two_x1 = mul_mod_u256(U256::from(2), p1.x, curve.p);
                if slope_sq >= two_x1 { (slope_sq - two_x1) % curve.p }
                else { curve.p - ((two_x1 - slope_sq) % curve.p) }
            };
            let y3 = {
                let x_diff = if p1.x >= x3 { p1.x - x3 }
                else { curve.p - ((x3 - p1.x) % curve.p) };
                let slope_mul = mul_mod_u256(slope, x_diff, curve.p);
                if slope_mul >= p1.y { (slope_mul - p1.y) % curve.p }
                else { curve.p - ((p1.y - slope_mul) % curve.p) }
            };
            Point { x: x3, y: y3 }
        } else { Point { x: U256::zero(), y: U256::zero() } }
    } else {
        let numerator = if p2.y >= p1.y { (p2.y - p1.y) % curve.p }
        else { curve.p - ((p1.y - p2.y) % curve.p) };
        let denominator = if p2.x >= p1.x { (p2.x - p1.x) % curve.p }
        else { curve.p - ((p1.x - p2.x) % curve.p) };
        
        if let Some(denom_inv) = mod_inverse_u256(denominator, curve.p) {
            let slope = mul_mod_u256(numerator, denom_inv, curve.p);
            let x3 = {
                let slope_sq = mul_mod_u256(slope, slope, curve.p);
                let sum = (p1.x + p2.x) % curve.p;
                if slope_sq >= sum { (slope_sq - sum) % curve.p }
                else { curve.p - ((sum - slope_sq) % curve.p) }
            };
            let y3 = {
                let x_diff = if p1.x >= x3 { p1.x - x3 }
                else { curve.p - ((x3 - p1.x) % curve.p) };
                let slope_mul = mul_mod_u256(slope, x_diff, curve.p);
                if slope_mul >= p1.y { (slope_mul - p1.y) % curve.p }
                else { curve.p - ((p1.y - slope_mul) % curve.p) }
            };
            Point { x: x3, y: y3 }
        } else { Point { x: U256::zero(), y: U256::zero() } }
    }
}

fn ecc_scalar_mult(k: U256, point: &Point, curve: &EccCurve) -> Point {
    if k.is_zero() { return Point { x: U256::zero(), y: U256::zero() }; }
    
    let mut result = Point { x: U256::zero(), y: U256::zero() };
    let mut addend = point.clone();
    let mut scalar = k;
    
    while !scalar.is_zero() {
        if scalar & U256::one() == U256::one() {
            result = ecc_point_add(&result, &addend, curve);
        }
        addend = ecc_point_add(&addend, &addend, curve);
        scalar = scalar >> 1;
    }
    result
}

fn generate_rsa(p: U512, q: U512, seed: &mut U512) -> RSA {
    let modulus = if let Some(m) = p.checked_mul(q) { 
        m 
    } else { 
        mul_mod(p, q, U512::MAX) 
    };
    
    let p_minus_1 = p - U512::one();
    let q_minus_1 = q - U512::one();
    let totient = if let Some(t) = p_minus_1.checked_mul(q_minus_1) { t }
    else { mul_mod(p_minus_1, q_minus_1, U512::MAX) };
    
    let exponent = U256::from(65537);
    let private_key = mod_inverse(u256_to_u512(exponent), totient).unwrap_or_else(|| {
        mod_inverse(U512::from(3), totient).unwrap_or(U512::one())
    });
    
    let max_session = if modulus > U512::from(3) { modulus - U512::from(3) } 
    else { U512::from(100) };
    let session_key = simple_rand(seed, max_session) + U512::from(2);
    
    RSA { session_key, modulus, totient, exponent, private_key }
}

fn generate_ecc(prime: U512, rsa_private_key: U512, seed: &mut U512) -> ECC {
    let prime_u256 = u512_to_u256(prime);
    let a = U256::from(2); let b = U256::from(3);
    
    let gen_x = if prime > U512::one() {
        u512_to_u256(simple_rand(seed, prime - U512::one()) + U512::one())
    } else { U256::from(2) };
    
    let gen_y = if prime > U512::one() {
        u512_to_u256(simple_rand(seed, prime - U512::one()) + U512::one())
    } else { U256::from(2) };
    
    let generator = Point { x: gen_x, y: gen_y };
    let curve = EccCurve { a, b, p: prime_u256, generator: generator.clone() };
    
    let private_key1 = u512_to_u256(rsa_private_key % prime);
    let public_key1 = ecc_scalar_mult(private_key1, &generator, &curve);
    
    let private_key2 = if prime > U512::one() {
        u512_to_u256(simple_rand(seed, prime - U512::one()) + U512::one())
    } else { U256::from(3) };
    let public_key2 = ecc_scalar_mult(private_key2, &generator, &curve);
    
    ECC { curve, private_key1, public_key1, private_key2, public_key2 }
}

fn next_safe_prime(n: U512, bits: u16, min: U512, max: U512) -> U512 {
    const MAX_ITER: u32 = if cfg!(debug_assertions) { 500_000 } else { 5_000_000 };
    
    let q_min = min >> 1;
    let q_max = max >> 1;
    let mut q = (n >> 1) | U512::one();
    if q < q_min { q = q_min | U512::one(); }
    
    let mut iter = 0u32;
    loop {
        if q > q_max { q = q_min | U512::one(); }
        
        let mut q_divisible = false;
        for &small_p in &SMALL_PRIMES {
            if q % small_p == U512::zero() && q != small_p {
                q_divisible = true;
                break;
            }
        }
        
        if !q_divisible {
            let p = (q << 1) + U512::one(); // p = 2q + 1
            
            let mut p_divisible = false;
            for &small_p in &SMALL_PRIMES {
                if p % small_p == U512::zero() && p != small_p {
                    p_divisible = true; break;
                }
            }
            
            if !p_divisible && p >= min && p <= max {
                if is_prime(q, bits - 1) {
                    if is_prime(p, bits) { return p; }
                }
            }
        }
        
        q = q + U512::from(2);
        iter += 1;
        if iter >= MAX_ITER {
            eprintln!("[ERROR] Timeout: Could not find a safe prime for {} bits after {} iterations, exiting...", bits, MAX_ITER);
            std::process::exit(1);
        }
    }
}

fn fmt_u512(v: U512) -> String {
    format!("U512([{}, {}, {}, {}, {}, {}, {}, {}])", v.0[0], v.0[1], v.0[2], v.0[3], v.0[4], v.0[5], v.0[6], v.0[7])
}

fn fmt_u256(v: U256) -> String {
    format!("U256([{}, {}, {}, {}])", v.0[0], v.0[1], v.0[2], v.0[3])
}

fn fmt_point(p: &Point) -> String {
    format!("Point {{ x: {}, y: {} }}", fmt_u256(p.x), fmt_u256(p.y))
}

fn fmt_curve(c: &EccCurve) -> String {
    format!("EccCurve {{\n                a: CURVE_A, b: CURVE_B,\n                p: {},\n                generator: {}\n            }}", fmt_u256(c.p), fmt_point(&c.generator))
}

fn fmt_rsa(r: &RSA) -> String {
    format!("RSA {{\n            session_key: {},\n            modulus: {},\n            totient: {},\n            exponent: {},\n            private_key: {}\n        }}", 
        fmt_u512(r.session_key), fmt_u512(r.modulus), fmt_u512(r.totient), fmt_u256(r.exponent), fmt_u512(r.private_key))
}

fn fmt_ecc(e: &ECC) -> String {
    format!("ECC {{\n            curve: {},\n            private_key1: {},\n            public_key1: {},\n            private_key2: {},\n            public_key2: {}\n        }}", 
        fmt_curve(&e.curve), fmt_u256(e.private_key1), fmt_point(&e.public_key1), fmt_u256(e.private_key2), fmt_point(&e.public_key2))
}

fn main() {
    let file = File::create("./lookup.rs").expect("Failed to create lookup.rs");
    let mut writer = BufWriter::new(file);
    
    writeln!(writer, "// This file is auto-generated. Do not edit manually.").unwrap();
    writeln!(writer, "use primitive_types::{{U256, U512}};\n").unwrap();
    writeln!(writer, "// Curve parameters: y^2 = x^3 + ax + b (mod p)").unwrap();
    writeln!(writer, "pub const CURVE_A: U256 = U256([2, 0, 0, 0]);").unwrap();
    writeln!(writer, "pub const CURVE_B: U256 = U256([3, 0, 0, 0]);\n").unwrap();
    writeln!(writer, "#[derive(Clone)]").unwrap();
    writeln!(writer, "pub struct RSA {{ pub session_key: U512, pub modulus: U512, pub totient: U512, pub exponent: U256, pub private_key: U512}}\n").unwrap();
    writeln!(writer, "#[derive(Clone)]").unwrap();
    writeln!(writer, "pub struct Point {{ pub x: U256, pub y: U256}}\n").unwrap();
    writeln!(writer, "#[derive(Clone)]").unwrap();
    writeln!(writer, "pub struct EccCurve {{ pub a: U256, pub b: U256, pub p: U256,  pub generator: Point}}\n").unwrap();
    writeln!(writer, "#[derive(Clone)]").unwrap();
    writeln!(writer, "pub struct ECC {{ pub curve: EccCurve, pub private_key1: U256, pub public_key1: Point, pub private_key2: U256, pub public_key2: Point}}\n").unwrap();
    writeln!(writer, "#[derive(Clone)]").unwrap();
    writeln!(writer, "pub struct KeySize {{ pub bits: u16, pub rsa: RSA, pub ecc: ECC}}\n").unwrap();
    
    const TOTAL_TRIALS: usize = ((PRIME_RANGE[1] - PRIME_RANGE[0]) / 8 + 1) as usize * TRIALS as usize;
    let mut lookup: [Option<KeySize>; TOTAL_TRIALS] = [const { None }; TOTAL_TRIALS];
    let mut index = 0usize;
    
    let seed_base = U512::from(SEED);
    for bits in PRIME_RANGE[0]..=PRIME_RANGE[1] {
        if bits % 8 != 0 { continue; }
        let min = U512::one() << (bits - 1);
        let max = (U512::one() << bits) - U512::one();
        let bits_u512 = U512::from(bits);
        
        let seed_mult = if let Some(product) = seed_base.checked_mul(bits_u512) {
            product.overflowing_mul(U512::from(6364136223846793005u64)).0
        } else {
            mul_mod(seed_base, bits_u512, U512::MAX)
        };
        let seed_add = seed_mult.overflowing_add(U512::from(1442695040888963407u64)).0;
        let range_size = max - min + U512::one();
        let mut candidate = min + (seed_add % range_size);
        candidate = candidate | min;

        let mut seed2 = U512::from(SEED) + U512::from(bits as u64);
        
        for _ in 0..TRIALS {
            let p1 = next_safe_prime(candidate, bits, min, max);
            candidate = p1 + U512::from(4);
            let p2 = next_safe_prime(candidate, bits, min, max);
            candidate = p2 + U512::from(4);
            let p3 = next_safe_prime(candidate, bits, min, max);
            candidate = p3 + U512::from(4);
            
            let rsa = generate_rsa(p1, p2, &mut seed2);
            let ecc = generate_ecc(p3, rsa.private_key, &mut seed2);
            lookup[index] = Some(KeySize { bits, rsa, ecc });
            index += 1;
        }
        println!("{} bits trials generated", bits);
    }
        
    // write lookup array to file
    writeln!(writer, "// Array of all KeySize structs\npub const KEYSIZES: [KeySize; {}] = [", index).unwrap();
    
    for i in 0..TOTAL_TRIALS {
        if let Some(ref keysize) = lookup[i] {
            writeln!(writer, "    KeySize {{").unwrap();
            writeln!(writer, "        bits: {},", keysize.bits).unwrap();
            writeln!(writer, "        rsa: {},", fmt_rsa(&keysize.rsa)).unwrap();
            writeln!(writer, "        ecc: {}", fmt_ecc(&keysize.ecc)).unwrap();
            if i < index - 1 {
                writeln!(writer, "    }},").unwrap();
            } else {
                writeln!(writer, "    }}").unwrap();
            }
        }
    }
    
    writeln!(writer, "];").unwrap();
    writer.flush().unwrap();
    println!("Generated lookup.rs successfully with {} KeySize structs!", index);
}
