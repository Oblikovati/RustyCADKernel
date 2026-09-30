// Rust translation of FoundationClasses/TKernel/Standard/Standard_HashUtils.hxx and .lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `opencascade::FNVHash`: FNV-1a with OCCT's plain-`char` quirk.
//!
//! The header warns that the loop reads `const char`, not `unsigned char`: where `char`
//! is signed (x86-64 Linux, the oracle profile) a byte >= 0x80 is sign-extended before
//! the XOR, so the result is not textbook FNV-1a. The port converts each byte through
//! `c_char`, which has the platform's signedness, and keeps the quirk.

use super::{HashWord, object_bytes};
use bytemuck::NoUninit;
use std::ffi::c_char;

/// `static_cast<uint64_t>(char)`: sign- or zero-extension, following the platform's `char`.
fn widen_char(byte: u8) -> u64 {
    // `as` from a signed `c_char` sign-extends; from an unsigned one it zero-extends.
    #[allow(clippy::unnecessary_cast)]
    let widened: u64 = byte as c_char as i64 as u64;
    widened
}

/// `FNVHash1A(const void*, int, uint32_t)`: 32-bit FNV-1a over `char` values.
///
/// ```
/// use rustycad_tkernel::standard::hashing::fnv_hash::fnv_hash_1a;
/// assert_eq!(fnv_hash_1a(b"a", 2_166_136_261), 0xe40c_292c);
/// ```
pub fn fnv_hash_1a(key: &[u8], seed: u32) -> u32 {
    key.iter().fold(seed, |hash, &byte| {
        (hash ^ widen_char(byte) as u32).wrapping_mul(16_777_619)
    })
}

/// `FNVHash64A(const void*, int, uint64_t)`: 64-bit FNV-1a over `char` values.
///
/// ```
/// use rustycad_tkernel::standard::hashing::fnv_hash::fnv_hash_64a;
/// assert_eq!(fnv_hash_64a(b"a", 14_695_981_039_346_656_037), 0xaf63_dc4c_8601_ec8c);
/// ```
pub fn fnv_hash_64a(key: &[u8], seed: u64) -> u64 {
    key.iter().fold(seed, |hash, &byte| {
        (hash ^ widen_char(byte)).wrapping_mul(1_099_511_628_211)
    })
}

/// `FNVHash::hash_combine(const T1&, int, T)`: both `enable_if` overloads; an 8-byte `W`
/// runs [`fnv_hash_64a`], any other width [`fnv_hash_1a`]. The C++ defaults are
/// `length = sizeof(T1)` and `seed = optimalSeed<T>()`.
///
/// # Panics
/// When `length` exceeds the byte size of `values` (C++ would read past the object).
///
/// ```
/// use rustycad_tkernel::standard::hashing::fnv_hash::{hash_combine, optimal_seed};
/// let code: u64 = hash_combine(b"a", 1, optimal_seed());
/// assert_eq!(code, 0xaf63_dc4c_8601_ec8c);
/// ```
pub fn hash_combine<W: HashWord, T: NoUninit>(values: &[T], length: usize, seed: W) -> W {
    W::fnv(object_bytes(values, length), seed)
}

/// `FNVHash::optimalSeed<T>()`: the FNV offset basis of the word width.
///
/// ```
/// use rustycad_tkernel::standard::hashing::fnv_hash::optimal_seed;
/// assert_eq!(optimal_seed::<u32>(), 2_166_136_261);
/// ```
pub fn optimal_seed<W: HashWord>() -> W {
    W::FNV_SEED
}
