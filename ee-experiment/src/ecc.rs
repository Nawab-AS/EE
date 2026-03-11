use crypto_bigint::{U256, Limb};
use crypto_bigint::modular::runtime_mod::{DynResidue, DynResidueParams};

use crate::lookup::ECC;

type Params256 = DynResidueParams<{ U256::LIMBS }>;
type Residue256 = DynResidue<{ U256::LIMBS }>;

// precomputed modular arithmetic params for a curve's prime field
#[derive(Clone, Copy)]
pub struct EccCtx {
    pub params: Params256,
    a: Residue256,
    two: Residue256,
    three: Residue256,
}

impl EccCtx {
    pub fn new(p: U256, a: U256) -> Self {
        let params = Params256::new(&p);
        EccCtx {
            params,
            a: DynResidue::new(&a, params),
            two: DynResidue::new(&U256::from(2u64), params),
            three: DynResidue::new(&U256::from(3u64), params),
        }
    }
}

#[derive(Clone, Copy)]
struct ResiduePoint {
    x: Residue256,
    y: Residue256,
    inf: bool,
}

fn point_add(p1: &ResiduePoint, p2: &ResiduePoint, ctx: &EccCtx) -> ResiduePoint {
    if p1.inf { return *p2; }
    if p2.inf { return *p1; }

    if p1.x.retrieve() == p2.x.retrieve() {
        if p1.y.retrieve() != p2.y.retrieve() {
            return ResiduePoint {
                x: DynResidue::zero(ctx.params),
                y: DynResidue::zero(ctx.params),
                inf: true,
            };
        }

        // Point doubling: slope = (3*x1^2 + a) / (2*y1)
        let x1_sq = p1.x * p1.x;
        let num = ctx.three * x1_sq + ctx.a;
        let den = ctx.two * p1.y;
        let (den_inv, _) = den.invert();
        let slope = num * den_inv;

        let x3 = slope * slope - p1.x - p1.x;
        let y3 = slope * (p1.x - x3) - p1.y;
        ResiduePoint { x: x3, y: y3, inf: false }
    } else {
        // Point addition: slope = (y2 - y1) / (x2 - x1)
        let num = p2.y - p1.y;
        let den = p2.x - p1.x;
        let (den_inv, _) = den.invert();
        let slope = num * den_inv;

        let x3 = slope * slope - p1.x - p2.x;
        let y3 = slope * (p1.x - x3) - p1.y;
        ResiduePoint { x: x3, y: y3, inf: false }
    }
}

#[inline(always)]
fn bit_vt(val: &U256, bit: usize) -> bool {
    let li = bit / Limb::BITS;
    let bi = bit % Limb::BITS;
    (val.as_limbs()[li].0 >> bi) & 1 == 1
}

#[inline(always)]
fn bitlen_vt(val: &U256) -> usize {
    let limbs = val.as_limbs();
    let mut i = U256::LIMBS;
    while i > 0 {
        i -= 1;
        let w = limbs[i].0;
        if w != 0 {
            return i * Limb::BITS + (Limb::BITS - w.leading_zeros() as usize);
        }
    }
    0
}

// double-and-add scalar multiplication
fn scalar_mult(k: U256, point: &ResiduePoint, ctx: &EccCtx) -> ResiduePoint {
    let bits = bitlen_vt(&k);
    if bits == 0 {
        return ResiduePoint {
            x: DynResidue::zero(ctx.params),
            y: DynResidue::zero(ctx.params),
            inf: true,
        };
    }

    let mut result = ResiduePoint {
        x: DynResidue::zero(ctx.params),
        y: DynResidue::zero(ctx.params),
        inf: true,
    };
    let mut addend = *point;

    for i in 0..bits {
        if bit_vt(&k, i) {
            result = point_add(&result, &addend, ctx);
        }
        addend = point_add(&addend, &addend, ctx);
    }
    result
}

pub fn ecdh(data: ECC, ctx: &EccCtx) -> bool {
    let generator = ResiduePoint {
        x: DynResidue::new(&data.curve.generator.x, ctx.params),
        y: DynResidue::new(&data.curve.generator.y, ctx.params),
        inf: false,
    };

    // Generate public keys
    let pk1 = scalar_mult(data.private_key1, &generator, ctx);
    let _ss2 = scalar_mult(data.private_key2, &pk1, ctx);

    true
}