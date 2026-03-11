// (https://csrc.nist.gov/csrc/media/projects/cryptographic-module-validation-program/documents/fips140-2/fips1402ig.pdf)
// calculated from "Implementation Guidance for FIPS 140-2 and the Cryptographic
// Module Validation Program", section 7.5, page 126, equation 1 & 2

pub const ECC_V_RSA: [(u16, u16); 13] = [ // in order of security bits, step of 1 byte
    (16, 32),
    (32, 64),
    (48, 112),
    (64, 176),
    (80, 264),
    (96, 368),
    (112, 496),
    (128, 648),
    (144, 824),
    (160, 1024),
    (176, 1256),
    (192, 1520),
    (208, 1806)
];