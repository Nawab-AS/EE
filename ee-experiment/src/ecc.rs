// Non-standard time implementation of ECH, not for production use

use primitive_types::U256;

use crate::lookup::{ECC, EccCurve, Point};
use crate::helper::{mul_mod_u256, mod_inv_u256};

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
        
        let denom_inv = mod_inv_u256(denominator, curve.p);
        let slope = mul_mod_u256(U256::from(numerator), denom_inv, curve.p);
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
    } else {
        let numerator = if p2.y >= p1.y { (p2.y - p1.y) % curve.p }
        else { curve.p - ((p1.y - p2.y) % curve.p) };
        let denominator = if p2.x >= p1.x { (p2.x - p1.x) % curve.p }
        else { curve.p - ((p1.x - p2.x) % curve.p) };
        
        let denom_inv = mod_inv_u256(denominator, curve.p);
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


#[allow(non_snake_case)]
pub fn ECDH(data: ECC) -> bool{
    // generate public keys
    let public_key1 = ecc_scalar_mult(data.private_key1, &data.curve.generator, &data.curve);
    let public_key2 = ecc_scalar_mult(data.private_key2, &data.curve.generator, &data.curve);

    // generate shared secrets
    let shared_secret1 = ecc_scalar_mult(data.private_key1, &public_key2, &data.curve).x;
    let shared_secret2 = ecc_scalar_mult(data.private_key2, &public_key1, &data.curve).x;

    shared_secret1 == shared_secret2
}