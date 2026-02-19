use primitive_types::U512;

use crate::helper::{mul_mod, mod_inv};

pub fn pow_mod(mut base: U512, mut exp: U512, m: U512) -> U512 {
    if m.is_zero() { return U512::zero();}
    if m == U512::one() { return U512::zero(); }
    
    let mut res = U512::one();
    base = base % m;

    while !exp.is_zero() {
        if exp.bit(0) {
            res = mul_mod(res, base, m);
        }
        base = mul_mod(base, base, m);
        exp >>= 1;
    }
    res
}

#[allow(non_snake_case)]
pub fn KEY_TRANSPORT(rsa: crate::lookup::RSA) -> bool {
    // generate private key and encrypted session secret
    let private_key = mod_inv(U512::from(rsa.exponent), rsa.totient);
    let encrypted_session_key = pow_mod(rsa.session_key, U512::from(rsa.exponent), rsa.modulus);

    // decrypt session key
    let session_key = pow_mod(encrypted_session_key, private_key, rsa.modulus);

    session_key == rsa.session_key
}