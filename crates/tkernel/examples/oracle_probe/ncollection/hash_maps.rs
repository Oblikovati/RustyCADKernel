//! NCollection hash maps probe operations (twin of `tkernel_ncollection_hash_maps_probe.cxx`).
//!
//! Each map request is a script of single-letter operations; every operation prints its
//! result (or `throw <ExceptionType>`), then the map's bucket count, extent and full
//! cursor traversal, which exposes the exact OCCT bucket order.
use rustycad_tkernel::ncollection::{DefaultHashPolicy, HashPolicy, next_map_prime};
use rustycad_tkernel::standard::Failure;
use std::str::SplitWhitespace;

mod indexed;
mod keyed;

/// The output of a hash maps operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    let output = match op {
        "ncoll_hash_maps_prime" => next_map_prime(size(input)).to_string(),
        "ncoll_hash_maps_hash" => hash(input),
        "ncoll_hash_maps_datamap" => keyed::data_map(input),
        "ncoll_hash_maps_map" => keyed::map(input),
        "ncoll_hash_maps_algo" => keyed::algo(input),
        "ncoll_hash_maps_indexed" => indexed::indexed_map(input),
        "ncoll_hash_maps_indexed_data" => indexed::indexed_data_map(input),
        "ncoll_hash_maps_double" => indexed::double_map(input),
        _ => return None,
    };
    Some(output)
}

/// The next integer field, read like `std::istream >> int`.
pub(super) fn int(input: &mut SplitWhitespace<'_>) -> i32 {
    input.next().expect("int field").parse().expect("int input")
}

/// The next size field, read like `std::istream >> size_t`.
pub(super) fn size(input: &mut SplitWhitespace<'_>) -> usize {
    input
        .next()
        .expect("size field")
        .parse()
        .expect("size input")
}

/// A `bool` streamed by the native probe.
pub(super) fn flag(value: bool) -> String {
    u8::from(value).to_string()
}

/// The native `hm_guard`: the value, or `throw <type>` when the call raised, then a space.
pub(super) fn guard<T: std::fmt::Display>(result: Result<T, Failure>) -> String {
    match result {
        Ok(value) => format!("{value} "),
        Err(failure) => format!("throw {} ", failure.exception_type()),
    }
}

/// `NCollection_DefaultHasher<T>` applied to a value converted like `static_cast<T>(long long)`.
fn hash(input: &mut SplitWhitespace<'_>) -> String {
    let kind = input.next().expect("hash type");
    if kind == "double" {
        let value: f64 = input.next().expect("double").parse().expect("double input");
        return hashed(value);
    }
    let value: i64 = input.next().expect("value").parse().expect("integer input");
    match kind {
        "bool" => hashed(value != 0),
        "char" | "schar" => hashed(value as i8),
        "uchar" => hashed(value as u8),
        // Linux wchar_t is a signed 32-bit type; char32_t is unsigned.
        "wchar" | "int" => hashed(value as i32),
        "char16" | "ushort" => hashed(value as u16),
        "char32" | "uint" => hashed(value as u32),
        "short" => hashed(value as i16),
        "long" | "llong" => hashed(value),
        "ulong" | "ullong" => hashed(value as u64),
        other => panic!("unknown hash type {other}"),
    }
}

fn hashed<K>(key: K) -> String
where
    DefaultHashPolicy: HashPolicy<K>,
{
    let policy = DefaultHashPolicy;
    format!("{} {}", policy.hash(&key), flag(policy.equal(&key, &key)))
}
