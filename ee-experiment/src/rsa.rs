use crypto_bigint::{U256, U1024, U2048, NonZero, Limb};
use crypto_bigint::modular::runtime_mod::{DynResidue, DynResidueParams};

type Params2048 = DynResidueParams<{ U2048::LIMBS }>;
type Residue2048 = DynResidue<{ U2048::LIMBS }>;

#[inline(always)]
fn bit_vt(val: &U2048, bit: usize) -> bool {
    let li = bit / Limb::BITS;
    let bi = bit % Limb::BITS;
    (val.as_limbs()[li].0 >> bi) & 1 == 1
}

#[inline(always)]
fn bitlen_vt(val: &U2048) -> usize {
    let limbs = val.as_limbs();
    let mut i = U2048::LIMBS;
    while i > 0 {
        i -= 1;
        let w = limbs[i].0;
        if w != 0 {
            return i * Limb::BITS + (Limb::BITS - w.leading_zeros() as usize);
        }
    }
    0
}

// widen U1024 -> U2048 by copying limbs
fn widen_u1024(v: U1024) -> U2048 {
    let src = v.as_limbs();
    let mut limbs = [Limb::ZERO; U2048::LIMBS];
    let mut i = 0;
    while i < U1024::LIMBS {
        limbs[i] = src[i];
        i += 1;
    }
    U2048::from(limbs)
}

// widen U256 -> U2048
fn widen_u256(v: U256) -> U2048 {
    let src = v.as_limbs();
    let mut limbs = [Limb::ZERO; U2048::LIMBS];
    let mut i = 0;
    while i < U256::LIMBS {
        limbs[i] = src[i];
        i += 1;
    }
    U2048::from(limbs)
}

// extended GCD for modular inverse
// can't use DynResidue here because totient is even
fn extended_gcd(a: U2048, b: U2048) -> (U2048, U2048, bool) {
    if b == U2048::ZERO { return (a, U2048::ONE, false); }

    let mut old_r = a;
    let mut r = b;
    let mut old_s = U2048::ONE;
    let mut s = U2048::ZERO;
    let mut old_s_neg = false;
    let mut s_neg = false;

    while r != U2048::ZERO {
        let nz_r = NonZero::new(r).unwrap();
        let (q, rem) = old_r.div_rem(&nz_r);
        old_r = r;
        r = rem;

        let prod = q.wrapping_mul(&s);
        let tmp_s = s;
        let tmp_s_neg = s_neg;
        if old_s_neg == s_neg {
            if old_s >= prod {
                s = old_s.wrapping_sub(&prod);
                s_neg = old_s_neg;
            } else {
                s = prod.wrapping_sub(&old_s);
                s_neg = !old_s_neg;
            }
        } else {
            s = old_s.wrapping_add(&prod);
            s_neg = old_s_neg;
        }
        old_s = tmp_s;
        old_s_neg = tmp_s_neg;
    }
    (old_r, old_s, old_s_neg)
}

fn mod_inv(a: U2048, m: U2048) -> U2048 {
    let (gcd, x, x_neg) = extended_gcd(a, m);
    if gcd != U2048::ONE { crate::exit(); }
    let nz_m = NonZero::new(m).unwrap();
    let (_, rem) = x.div_rem(&nz_m);
    if x_neg { m.wrapping_sub(&rem) } else { rem }
}

pub struct RsaCtx {
    params: Params2048,
    totient: U2048,
}

impl RsaCtx {
    pub fn new(rsa: &crate::lookup::RSA) -> Self {
        let p = widen_u1024(rsa.p);
        let q = widen_u1024(rsa.q);
        let n = p.wrapping_mul(&q);
        let totient = p.wrapping_sub(&U2048::ONE).wrapping_mul(&q.wrapping_sub(&U2048::ONE));
        let params = Params2048::new(&n);
        RsaCtx { params, totient }
    }
}

// variable-time modular exponentiation (square-and-multiply)
fn pow_vartime(base: Residue2048, exp: &U2048, params: Params2048) -> Residue2048 {
    let bits = bitlen_vt(exp);
    if bits == 0 {
        return DynResidue::new(&U2048::ONE, params);
    }

    let mut result = DynResidue::new(&U2048::ONE, params);
    let mut acc = base;

    for i in 0..bits {
        if bit_vt(exp, i) {
            result = result * acc;
        }
        acc = acc * acc;
    }
    result
}

pub fn key_transport(rsa: crate::lookup::RSA, ctx: &RsaCtx) -> bool {
    let e = widen_u256(rsa.exponent);

    let d = mod_inv(e, ctx.totient);

    // encrypt: c = session_key^e mod n
    let base = DynResidue::new(&rsa.session_key, ctx.params);
    let encrypted = pow_vartime(base, &e, ctx.params);

    // decrypt: m = c^d mod n
    let decrypted = pow_vartime(encrypted, &d, ctx.params);

    decrypted.retrieve() == rsa.session_key
}