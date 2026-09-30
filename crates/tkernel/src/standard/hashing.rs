// Rust translation of FoundationClasses/TKernel/Standard/Standard_HashUtils.hxx and .lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `opencascade::hash`, `hashBytes` and `hash_combine` with the Murmur and FNV-1a engines.
//!
//! The C++ templates hash the *object representation* of a value (`&theValue`,
//! `sizeof(T1)` bytes, or `theLen` bytes from that address). The port models that with
//! `bytemuck::NoUninit`, which admits exactly the types whose bytes are all initialized
//! (no padding), so the byte view equals the C++ one on the same platform. The result
//! type `T` of the templates (default `size_t`) is the [`HashWord`] parameter: an 8-byte
//! word selects the 64-bit engine, any other width the 32-bit one, as `enable_if` does.

pub mod fnv_hash;
pub mod murmur_hash;

use bytemuck::NoUninit;

/// A result/seed word of the hash templates: `u32`, `u64` or `usize` (`size_t`).
pub trait HashWord: Copy {
    /// `MurmurHash::optimalSeed<T>()`.
    const MURMUR_SEED: Self;
    /// `FNVHash::optimalSeed<T>()`.
    const FNV_SEED: Self;
    /// `MurmurHash::hash_combine` over raw bytes with this word's engine.
    fn murmur(bytes: &[u8], seed: Self) -> Self;
    /// `FNVHash::hash_combine` over raw bytes with this word's engine.
    fn fnv(bytes: &[u8], seed: Self) -> Self;
}

impl HashWord for u64 {
    const MURMUR_SEED: Self = 0xA329_F1D3_A586;
    const FNV_SEED: Self = 14_695_981_039_346_656_037;
    fn murmur(bytes: &[u8], seed: Self) -> Self {
        murmur_hash::murmur_hash_64a(bytes, seed)
    }
    fn fnv(bytes: &[u8], seed: Self) -> Self {
        fnv_hash::fnv_hash_64a(bytes, seed)
    }
}

impl HashWord for u32 {
    const MURMUR_SEED: Self = 0xc70f_6907;
    const FNV_SEED: Self = 2_166_136_261;
    fn murmur(bytes: &[u8], seed: Self) -> Self {
        murmur_hash::murmur_hash_2a(bytes, seed)
    }
    fn fnv(bytes: &[u8], seed: Self) -> Self {
        fnv_hash::fnv_hash_1a(bytes, seed)
    }
}

// `size_t` follows the pointer width, exactly as `sizeof(T) == 8` selects the engine in C++.
#[cfg(target_pointer_width = "64")]
type SizeWord = u64;
#[cfg(not(target_pointer_width = "64"))]
type SizeWord = u32;

impl HashWord for usize {
    const MURMUR_SEED: Self = SizeWord::MURMUR_SEED as usize;
    const FNV_SEED: Self = SizeWord::FNV_SEED as usize;
    fn murmur(bytes: &[u8], seed: Self) -> Self {
        SizeWord::murmur(bytes, seed as SizeWord) as usize
    }
    fn fnv(bytes: &[u8], seed: Self) -> Self {
        SizeWord::fnv(bytes, seed as SizeWord) as usize
    }
}

/// The first `length` bytes of the object representation of `values`, as `(&theValue, theLen)`.
///
/// Reading past the object is undefined behavior in C++; the port panics with the sizes.
pub(crate) fn object_bytes<T: NoUninit>(values: &[T], length: usize) -> &[u8] {
    let bytes: &[u8] = bytemuck::cast_slice(values);
    assert!(
        length <= bytes.len(),
        "hash length {length} exceeds the {}-byte object representation; expected 0..={}",
        bytes.len(),
        bytes.len()
    );
    &bytes[..length]
}

/// `opencascade::hash(const T1&)`: Murmur over all `sizeof(T1)` bytes with the optimal seed.
///
/// ```
/// use rustycad_tkernel::standard::hashing::hash;
/// let code: usize = hash(&42_i32);
/// assert_eq!(code, rustycad_tkernel::standard::hash_bytes(&42_i32.to_ne_bytes()));
/// ```
pub fn hash<W: HashWord, T: NoUninit>(value: &T) -> W {
    murmur_hash::hash_combine(std::slice::from_ref(value), size_of::<T>(), W::MURMUR_SEED)
}

/// `opencascade::hashBytes(const T1*, int)` for `size_t`: Murmur over `bytes` with the optimal seed.
///
/// A typed key array is passed as `bytemuck::cast_slice(keys)`, the bytes C++ reads from the pointer.
///
/// ```
/// let code: usize = rustycad_tkernel::standard::hash_bytes(&42_i32.to_ne_bytes());
/// assert_eq!(code, rustycad_tkernel::standard::hashing::hash(&42_i32));
/// ```
pub fn hash_bytes(bytes: &[u8]) -> usize {
    usize::murmur(bytes, usize::MURMUR_SEED)
}

/// `opencascade::hash_combine(const T1&, int, T)`: Murmur over the first `length` bytes of
/// `values` with an explicit `seed`; it chains hashes by passing a previous result as seed.
///
/// # Panics
/// When `length` exceeds the byte size of `values` (C++ would read past the object).
///
/// ```
/// use rustycad_tkernel::standard::hashing::{hash, hash_combine, HashWord};
/// let code: usize = hash_combine(&[7_u16], 2, usize::MURMUR_SEED);
/// assert_eq!(code, hash::<usize, u16>(&7));
/// ```
pub fn hash_combine<W: HashWord, T: NoUninit>(values: &[T], length: usize, seed: W) -> W {
    murmur_hash::hash_combine(values, length, seed)
}

/// Native-width Murmur used by the other TKernel ports; `murmur64(b"key", 0xA329_F1D3_A586)`.
pub(crate) fn murmur64(bytes: &[u8], seed: u64) -> u64 {
    murmur_hash::murmur_hash_64a(bytes, seed)
}

/// 32-bit Murmur used by the other TKernel ports; `murmur32(b"key", 0xc70f_6907)`.
pub(crate) fn murmur32(bytes: &[u8], seed: u32) -> u32 {
    murmur_hash::murmur_hash_2a(bytes, seed)
}
