use primitive_types::U512;

use crate::helper::{MontCtxU512, mod_inv};

/// Montgomery modular exponentiation with 4-bit windowed method.
fn pow_mod(base: U512, exp: U512, ctx: &MontCtxU512) -> U512 {
    if ctx.n == U512::one() { return U512::zero(); }

    let base_mont = ctx.to_mont(base % ctx.n);
    let one_mont = ctx.to_mont(U512::one());

    // Precompute table[i] = base^i in Montgomery form (i = 0..15)
    let mut table = [U512::zero(); 16];
    table[0] = one_mont;
    table[1] = base_mont;
    for i in 2..16 {
        table[i] = ctx.mont_mul(table[i - 1], base_mont);
    }

    let mut result = one_mont;
    let mut started = false;

    // Process exponent from most-significant nibble downward
    for i in (0..128).rev() {
        let nibble = ((exp >> (i * 4)).0[0] & 0xF) as usize;

        if started {
            // Square 4 times
            result = ctx.mont_mul(result, result);
            result = ctx.mont_mul(result, result);
            result = ctx.mont_mul(result, result);
            result = ctx.mont_mul(result, result);
            if nibble != 0 {
                result = ctx.mont_mul(result, table[nibble]);
            }
        } else if nibble != 0 {
            result = table[nibble];
            started = true;
        }
    }

    if !started { return U512::zero(); }
    ctx.from_mont(result)
}

#[allow(non_snake_case)]
pub fn KEY_TRANSPORT(rsa: crate::lookup::RSA) -> bool {
    let ctx = MontCtxU512::new(rsa.modulus);

    // Generate private key
    let private_key = mod_inv(U512::from(rsa.exponent), rsa.totient);

    // Encrypt session key: c = session_key^e mod n
    let encrypted = pow_mod(rsa.session_key, U512::from(rsa.exponent), &ctx);

    // Decrypt: m = c^d mod n
    let decrypted = pow_mod(encrypted, private_key, &ctx);

    decrypted == rsa.session_key
}