use primitive_types::{U512, U256};
use cortex_m::asm;

use crate::uprint;

pub fn exit() -> ! {
    uprint!("Exiting...\n");
    loop {}
}

fn u512_to_u256(v: U512) -> U256 {
    U256([v.0[0], v.0[1], v.0[2], v.0[3]])
}

// ============================================================
// Montgomery arithmetic core
// ============================================================

/// Compute -n0^{-1} mod 2^64 via Newton's method (n0 must be odd).
#[inline(always)]
fn compute_neg_inv_word(n0: u64) -> u64 {
    let mut inv: u64 = 1;
    for _ in 0..6 {
        inv = inv.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(inv)));
    }
    inv.wrapping_neg()
}

/// Compute R^2 mod n for R = 2^256 by repeated doubling.
fn compute_r2_mod_n_u256(n: U256) -> U256 {
    let mut r = U256::one();
    for _ in 0..512 {
        let top = r.bit(255);
        r = r << 1;
        if top || r >= n {
            r = r.overflowing_sub(n).0;
        }
    }
    r
}

/// Compute R^2 mod n for R = 2^512 by repeated doubling.
fn compute_r2_mod_n_u512(n: U512) -> U512 {
    let mut r = U512::one();
    for _ in 0..1024 {
        let top = r.bit(511);
        r = r << 1;
        if top || r >= n {
            r = r.overflowing_sub(n).0;
        }
    }
    r
}

// ============================================================
// Montgomery context for U256 (ECC)
// ============================================================

#[derive(Clone, Copy)]
pub struct MontCtxU256 {
    pub n: U256,
    n_inv: u64,
    r2: U256,
}

impl MontCtxU256 {
    pub fn new(n: U256) -> Self {
        MontCtxU256 {
            n,
            n_inv: compute_neg_inv_word(n.0[0]),
            r2: compute_r2_mod_n_u256(n),
        }
    }

    /// Convert to Montgomery form: aR mod N.
    #[inline(always)]
    pub fn to_mont(&self, a: U256) -> U256 {
        self.mont_mul(a, self.r2)
    }

    /// Convert from Montgomery form: aR^{-1} mod N.
    #[inline(always)]
    pub fn from_mont(&self, a: U256) -> U256 {
        self.mont_mul(a, U256::one())
    }

    /// CIOS Montgomery multiplication (4 limbs): a*b*R^{-1} mod N.
    #[inline(always)]
    pub fn mont_mul(&self, a: U256, b: U256) -> U256 {
        let al = &a.0;
        let bl = &b.0;
        let n = &self.n.0;
        let ni = self.n_inv;
        let mut t = [0u64; 6];

        for i in 0..4 {
            let mut c: u64 = 0;
            for j in 0..4 {
                let uv = t[j] as u128 + al[j] as u128 * bl[i] as u128 + c as u128;
                t[j] = uv as u64;
                c = (uv >> 64) as u64;
            }
            let uv = t[4] as u128 + c as u128;
            t[4] = uv as u64;
            t[5] = (uv >> 64) as u64;

            let m = t[0].wrapping_mul(ni);
            let uv = t[0] as u128 + m as u128 * n[0] as u128;
            c = (uv >> 64) as u64;
            for j in 1..4 {
                let uv = t[j] as u128 + m as u128 * n[j] as u128 + c as u128;
                t[j - 1] = uv as u64;
                c = (uv >> 64) as u64;
            }
            let uv = t[4] as u128 + c as u128;
            t[3] = uv as u64;
            t[4] = t[5].wrapping_add((uv >> 64) as u64);
            t[5] = 0;
        }

        let result = U256([t[0], t[1], t[2], t[3]]);
        if t[4] > 0 || result >= self.n {
            result.overflowing_sub(self.n).0
        } else {
            result
        }
    }

    #[inline(always)]
    pub fn mont_add(&self, a: U256, b: U256) -> U256 {
        let (s, overflow) = a.overflowing_add(b);
        if overflow || s >= self.n { s.overflowing_sub(self.n).0 } else { s }
    }

    #[inline(always)]
    pub fn mont_sub(&self, a: U256, b: U256) -> U256 {
        let (d, borrow) = a.overflowing_sub(b);
        if borrow { d.overflowing_add(self.n).0 } else { d }
    }
}

// ============================================================
// Montgomery context for U512 (RSA)
// ============================================================

#[derive(Clone, Copy)]
pub struct MontCtxU512 {
    pub n: U512,
    n_inv: u64,
    r2: U512,
}

impl MontCtxU512 {
    pub fn new(n: U512) -> Self {
        MontCtxU512 {
            n,
            n_inv: compute_neg_inv_word(n.0[0]),
            r2: compute_r2_mod_n_u512(n),
        }
    }

    #[inline(always)]
    pub fn to_mont(&self, a: U512) -> U512 {
        self.mont_mul(a, self.r2)
    }

    #[inline(always)]
    pub fn from_mont(&self, a: U512) -> U512 {
        self.mont_mul(a, U512::one())
    }

    /// CIOS Montgomery multiplication (8 limbs): a*b*R^{-1} mod N.
    #[inline(always)]
    pub fn mont_mul(&self, a: U512, b: U512) -> U512 {
        let al = &a.0;
        let bl = &b.0;
        let n = &self.n.0;
        let ni = self.n_inv;
        let mut t = [0u64; 10];

        for i in 0..8 {
            let mut c: u64 = 0;
            for j in 0..8 {
                let uv = t[j] as u128 + al[j] as u128 * bl[i] as u128 + c as u128;
                t[j] = uv as u64;
                c = (uv >> 64) as u64;
            }
            let uv = t[8] as u128 + c as u128;
            t[8] = uv as u64;
            t[9] = (uv >> 64) as u64;

            let m = t[0].wrapping_mul(ni);
            let uv = t[0] as u128 + m as u128 * n[0] as u128;
            c = (uv >> 64) as u64;
            for j in 1..8 {
                let uv = t[j] as u128 + m as u128 * n[j] as u128 + c as u128;
                t[j - 1] = uv as u64;
                c = (uv >> 64) as u64;
            }
            let uv = t[8] as u128 + c as u128;
            t[7] = uv as u64;
            t[8] = t[9].wrapping_add((uv >> 64) as u64);
            t[9] = 0;
        }

        let result = U512([t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7]]);
        if t[8] > 0 || result >= self.n {
            result.overflowing_sub(self.n).0
        } else {
            result
        }
    }
}

// ============================================================
// Modular inverse via extended Euclidean GCD
// ============================================================

fn extended_gcd(a: U512, b: U512) -> (U512, U512, bool) {
    if b.is_zero() { return (a, U512::one(), false); }

    let mut old_r = a;
    let mut r = b;
    let mut old_s = U512::one();
    let mut s = U512::zero();
    let mut old_s_neg = false;
    let mut s_neg = false;

    while !r.is_zero() {
        let q = old_r / r;
        let tmp_r = r;
        r = old_r - q * r;
        old_r = tmp_r;

        let tmp_s = s;
        let tmp_s_neg = s_neg;
        let prod = q * s;
        if old_s_neg == s_neg {
            if old_s >= prod { s = old_s - prod; s_neg = old_s_neg; }
            else { s = prod - old_s; s_neg = !old_s_neg; }
        } else {
            s = old_s + prod; s_neg = old_s_neg;
        }
        old_s = tmp_s;
        old_s_neg = tmp_s_neg;
    }
    (old_r, old_s, old_s_neg)
}

pub fn mod_inv(a: U512, m: U512) -> U512 {
    let (gcd, x, x_neg) = extended_gcd(a, m);
    if gcd != U512::one() { exit(); }
    if x_neg { m - (x % m) } else { x % m }
}

pub fn mod_inv_u256(a: U256, m: U256) -> U256 {
    u512_to_u256(mod_inv(U512::from(a), U512::from(m)))
}