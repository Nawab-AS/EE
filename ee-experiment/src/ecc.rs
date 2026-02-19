// Non-constant-time ECDH implementation — not for production use

use primitive_types::U256;

use crate::lookup::ECC;
use crate::helper::{MontCtxU256, mod_inv_u256};

/// Precomputed Montgomery constants for a curve's prime field.
#[derive(Clone, Copy)]
struct EccMontCtx {
    ctx: MontCtxU256,
    a_mont: U256,
    two_mont: U256,
    three_mont: U256,
}

/// A point whose coordinates are in Montgomery form.
#[derive(Clone, Copy)]
struct MontPoint { x: U256, y: U256 }

impl MontPoint {
    #[inline(always)]
    fn infinity() -> Self { MontPoint { x: U256::zero(), y: U256::zero() } }
    #[inline(always)]
    fn is_inf(&self) -> bool { self.x.is_zero() && self.y.is_zero() }
}

/// Modular inverse in Montgomery domain.
#[inline(always)]
fn mont_inv(a_mont: U256, mc: &EccMontCtx) -> U256 {
    let a = mc.ctx.from_mont(a_mont);
    let a_inv = mod_inv_u256(a, mc.ctx.n);
    mc.ctx.to_mont(a_inv)
}

fn ecc_point_add(p1: &MontPoint, p2: &MontPoint, mc: &EccMontCtx) -> MontPoint {
    if p1.is_inf() { return *p2; }
    if p2.is_inf() { return *p1; }

    let ctx = &mc.ctx;

    if p1.x == p2.x {
        if p1.y != p2.y { return MontPoint::infinity(); }

        // Point doubling: slope = (3*x1^2 + a) / (2*y1)
        let x1_sq = ctx.mont_mul(p1.x, p1.x);
        let num = ctx.mont_add(ctx.mont_mul(mc.three_mont, x1_sq), mc.a_mont);
        let den = ctx.mont_mul(mc.two_mont, p1.y);
        let slope = ctx.mont_mul(num, mont_inv(den, mc));

        let slope_sq = ctx.mont_mul(slope, slope);
        let x3 = ctx.mont_sub(slope_sq, ctx.mont_add(p1.x, p1.x));
        let y3 = ctx.mont_sub(ctx.mont_mul(slope, ctx.mont_sub(p1.x, x3)), p1.y);
        MontPoint { x: x3, y: y3 }
    } else {
        // Point addition: slope = (y2 - y1) / (x2 - x1)
        let num = ctx.mont_sub(p2.y, p1.y);
        let den = ctx.mont_sub(p2.x, p1.x);
        let slope = ctx.mont_mul(num, mont_inv(den, mc));

        let slope_sq = ctx.mont_mul(slope, slope);
        let x3 = ctx.mont_sub(ctx.mont_sub(slope_sq, p1.x), p2.x);
        let y3 = ctx.mont_sub(ctx.mont_mul(slope, ctx.mont_sub(p1.x, x3)), p1.y);
        MontPoint { x: x3, y: y3 }
    }
}

fn ecc_scalar_mult(k: U256, point: &MontPoint, mc: &EccMontCtx) -> MontPoint {
    if k.is_zero() { return MontPoint::infinity(); }
    let mut result = MontPoint::infinity();
    let mut addend = *point;
    let mut scalar = k;

    while !scalar.is_zero() {
        if scalar & U256::one() == U256::one() {
            result = ecc_point_add(&result, &addend, mc);
        }
        addend = ecc_point_add(&addend, &addend, mc);
        scalar = scalar >> 1;
    }
    result
}

#[allow(non_snake_case)]
pub fn ECDH(data: ECC) -> bool {
    // Build Montgomery context once for this curve prime
    let ctx = MontCtxU256::new(data.curve.p);
    let mc = EccMontCtx {
        ctx,
        a_mont: ctx.to_mont(data.curve.a),
        two_mont: ctx.to_mont(U256::from(2)),
        three_mont: ctx.to_mont(U256::from(3)),
    };

    // Convert generator into Montgomery form
    let generator = MontPoint {
        x: ctx.to_mont(data.curve.generator.x),
        y: ctx.to_mont(data.curve.generator.y),
    };

    // Generate public keys
    let pk1 = ecc_scalar_mult(data.private_key1, &generator, &mc);
    let pk2 = ecc_scalar_mult(data.private_key2, &generator, &mc);

    // Generate shared secrets (compare x-coordinate, still in Montgomery form)
    let ss1 = ecc_scalar_mult(data.private_key1, &pk2, &mc);
    let ss2 = ecc_scalar_mult(data.private_key2, &pk1, &mc);

    // Equal in Montgomery form ↔ equal in normal form
    ss1.x == ss2.x
}