use primitive_types::{U512, U256};
use cortex_m::asm;

use crate::uprint;

pub fn exit() -> ! {
    uprint!("Exiting...\n");
    loop {}
}

fn u512_to_u256(original: U512) -> U256 {
    U256([original.0[0], original.0[1], original.0[2], original.0[3]])
}

pub fn mul_mod(a: U512, b: U512, m: U512) -> U512 {
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

pub fn mul_mod_u256(a: U256, b: U256, m: U256) -> U256 {
    let original = mul_mod(U512::from(a), U512::from(b), U512::from(m));
    u512_to_u256(original)
}



// Extended Euclidian GCD for modular inverse
fn extended_gcd(a: U512, b: U512) -> (U512, U512, bool) {
    if b.is_zero() { return (a, U512::one(), false); }
    
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
    (old_r, old_s, old_s_neg)
}

pub fn mod_inv(a: U512, m: U512) -> U512 {
    let (gcd, x, x_neg) = extended_gcd(a, m);
    if gcd != U512::one() { // error
        exit();
    }
    if x_neg { m - (x % m) } else { x % m }
}

pub fn mod_inv_u256(a: U256, m: U256) -> U256 {
    let original = mod_inv(U512::from(a), U512::from(m));
    u512_to_u256(original)
}