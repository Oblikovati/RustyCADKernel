//! Standard_HashUtils, Standard_CString and Standard_CStringHasher cases
//! (twin of tkernel_standard_values_hash_probe.cxx).
use super::super::common::{next, next_bytes};
use super::scalar::{hex32, hex64};
use bytemuck::NoUninit;
use rustycad_tkernel::standard::c_string::{self, CStringHasher};
use rustycad_tkernel::standard::hash_bytes;
use rustycad_tkernel::standard::hashing::{self, fnv_hash, murmur_hash};
use std::str::SplitWhitespace;

/// A 64-bit seed read from 16 hex digits.
fn next_seed(input: &mut SplitWhitespace<'_>) -> u64 {
    u64::from_str_radix(input.next().expect("seed"), 16).expect("hex seed")
}

/// The C string a `std::string::c_str()` presents: the bytes up to the first NUL.
fn c_str(bytes: &[u8]) -> &[u8] {
    bytes.split(|&byte| byte == 0).next().unwrap_or_default()
}

/// "hash_bytes <seed> <hex>": both engines, hashBytes and Standard_CStringHasher.
pub fn hash_bytes_case(input: &mut SplitWhitespace<'_>) -> String {
    let seed = next_seed(input);
    let bytes = next_bytes(input);
    let seed32 = seed as u32;
    format!(
        "{} {} {} {} {} {} {}",
        hex64(murmur_hash::murmur_hash_64a(&bytes, seed)),
        hex32(murmur_hash::murmur_hash_2a(&bytes, seed32)),
        hex32(fnv_hash::fnv_hash_1a(&bytes, seed32)),
        hex64(fnv_hash::fnv_hash_64a(&bytes, seed)),
        hex64(hash_bytes(&bytes) as u64),
        hex64(CStringHasher.hash_code(c_str(&bytes)) as u64),
        u8::from(CStringHasher.is_equal(c_str(&bytes), b"abcd"))
    )
}

/// "hash_combine <seed> <len> <hex64>": both engines' hash_combine and the top-level one.
pub fn hash_combine_case(input: &mut SplitWhitespace<'_>) -> String {
    let seed = next_seed(input);
    let length = next::<usize>(input);
    let bytes = next_bytes(input);
    let seed32 = seed as u32;
    format!(
        "{} {} {} {} {}",
        hex64(murmur_hash::hash_combine(&bytes, length, seed)),
        hex32(murmur_hash::hash_combine(&bytes, length, seed32)),
        hex64(fnv_hash::hash_combine(&bytes, length, seed)),
        hex32(fnv_hash::hash_combine(&bytes, length, seed32)),
        hex64(hashing::hash_combine(&bytes, length, seed as usize) as u64)
    )
}

/// Every default-argument hash of one typed value read from its object bytes.
fn typed_hashes<T: NoUninit + bytemuck::AnyBitPattern>(bytes: &[u8]) -> String {
    let value: T = bytemuck::pod_read_unaligned(bytes);
    let values = std::slice::from_ref(&value);
    let size = size_of::<T>();
    format!(
        "{} {} {} {} {} {}",
        hex64(hashing::hash::<usize, T>(&value) as u64),
        hex32(hashing::hash::<u32, T>(&value)),
        hex64(murmur_hash::hash_combine(values, size, murmur_hash::optimal_seed::<usize>()) as u64),
        hex32(murmur_hash::hash_combine(
            values,
            size,
            murmur_hash::optimal_seed::<u32>()
        )),
        hex64(fnv_hash::hash_combine(values, size, fnv_hash::optimal_seed::<usize>()) as u64),
        hex32(fnv_hash::hash_combine(
            values,
            size,
            fnv_hash::optimal_seed::<u32>()
        ))
    )
}

/// "hash_value <type> <hex>": opencascade::hash and the defaulted hash_combine overloads.
pub fn hash_value_case(input: &mut SplitWhitespace<'_>) -> String {
    let kind = input.next().expect("value type");
    let bytes = next_bytes(input);
    match kind {
        "char" => typed_hashes::<std::ffi::c_char>(&bytes),
        "uint16" => typed_hashes::<u16>(&bytes),
        "int" => typed_hashes::<i32>(&bytes),
        "int64" => typed_hashes::<i64>(&bytes),
        _ => typed_hashes::<f64>(&bytes),
    }
}

/// "hash_seeds": optimalSeed of both engines for size_t, uint32_t and uint64_t.
pub fn hash_seeds_case(_: &mut SplitWhitespace<'_>) -> String {
    format!(
        "{} {} {} {} {} {}",
        hex64(murmur_hash::optimal_seed::<usize>() as u64),
        hex32(murmur_hash::optimal_seed::<u32>()),
        hex64(murmur_hash::optimal_seed::<u64>()),
        hex64(fnv_hash::optimal_seed::<usize>() as u64),
        hex32(fnv_hash::optimal_seed::<u32>()),
        hex64(fnv_hash::optimal_seed::<u64>())
    )
}

/// "c_string_equal <hex> <hex>".
pub fn c_string_equal_case(input: &mut SplitWhitespace<'_>) -> String {
    let first = next_bytes(input);
    let second = next_bytes(input);
    u8::from(c_string::is_equal(&first, &second)).to_string()
}

/// "atof <hex>" (ASCII requests only).
pub fn atof_case(input: &mut SplitWhitespace<'_>) -> String {
    let text = String::from_utf8(next_bytes(input)).expect("ASCII Atof request");
    hex64(c_string::atof(&text).to_bits())
}
