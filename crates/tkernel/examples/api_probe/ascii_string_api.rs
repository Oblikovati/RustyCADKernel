//! Rust side of the differential cases for TCollection_AsciiString, TCollection_HAsciiString and TCollection.
//!
//! Twin of tools/oracle/tkernel_ascii_string_api_probe.cxx: requests are
//! `<group> <member> <fields...>`, strings travel as hex and print as `<hex> <length>`,
//! and an error prints only its exception class.
mod edits;
mod h_probe;
mod queries;

use super::common::{Case, guarded, hex, next, next_bytes};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::str::SplitWhitespace;

/// Operation name and handler of every case this area answers.
pub const CASES: &[(&str, Case)] = &[
    ("ascii_make", queries::make_case),
    ("ascii_edit", edits::edit_case),
    ("ascii_query", queries::query_case),
    ("ascii_stream", queries::stream_case),
    ("ascii_h_make", h_probe::h_make_case),
    ("ascii_h_edit", h_probe::h_edit_case),
    ("ascii_h_query", h_probe::h_query_case),
    ("ascii_next_prime", next_prime_case),
];

type Input<'a> = SplitWhitespace<'a>;
type Outcome = Result<String, Failure>;

/// A string holding exactly `bytes`, embedded NULs included (the C++ side's `Copy(ptr, len)`).
fn make(bytes: &[u8]) -> AsciiString {
    AsciiString::from_raw(bytes)
}

/// `<hex> <Length()>`, as the C++ `show`.
fn show(text: &AsciiString) -> String {
    format!("{} {}", hex(text.as_bytes()), text.length())
}

/// "ERR <class>: <message>" becomes "ERR <class>".
fn kind_only(response: String) -> String {
    match response.strip_prefix("ERR ") {
        Some(rest) => format!("ERR {}", rest.split(':').next().unwrap_or_default()),
        None => response,
    }
}

/// `guarded` then `kind_only`, the response of every case.
fn respond(outcome: Outcome) -> String {
    kind_only(guarded(outcome))
}

/// The bytes a C++ `const char*` built from these bytes designates: through the first NUL.
fn c_prefix(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len())]
}

/// C++ `static_cast<int>(long long)`: the low 32 bits.
fn next_int(input: &mut Input<'_>) -> i32 {
    next::<i64>(input) as i32
}

/// C++ `static_cast<char>(int)`: the low byte.
fn next_char(input: &mut Input<'_>) -> u8 {
    next_int(input) as u8
}

fn next_flag(input: &mut Input<'_>) -> bool {
    next_int(input) != 0
}

fn next_real(input: &mut Input<'_>) -> f64 {
    let token = input.next().expect("real bits field");
    f64::from_bits(u64::from_str_radix(token, 16).expect("hex real bits"))
}

fn next_text(input: &mut Input<'_>) -> Vec<u8> {
    next_bytes(input)
}

/// "<count> <u32>...", terminated by a zero unit as the C++ `std::wstring::c_str()`.
fn next_wide(input: &mut Input<'_>) -> Vec<u32> {
    let count = next_int(input).max(0);
    let mut units: Vec<u32> = (0..count).map(|_| next::<u64>(input) as u32).collect();
    units.push(0);
    units
}

/// Hex of 4-digit UTF-16 units, copied through the first zero unit.
fn next_extended(input: &mut Input<'_>) -> ExtendedString {
    let token = input.next().expect("utf-16 field");
    let units: Vec<u16> = if token == "-" {
        Vec::new()
    } else {
        (0..token.len() / 4)
            .map(|i| u16::from_str_radix(&token[4 * i..4 * i + 4], 16).expect("hex unit"))
            .collect()
    };
    ExtendedString::from_utf16(&units)
}

fn real_bits(value: f64) -> String {
    format!("{:x}", value.to_bits())
}

fn flag(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

/// "ascii_next_prime <int>": `TCollection::NextPrimeForMap`.
fn next_prime_case(input: &mut Input<'_>) -> String {
    rustycad_tkernel::tcollection::next_prime_for_map(next_int(input)).to_string()
}

/// Finds `<member>` in a probe table; an unknown name is a probe bug.
fn member<T: Copy>(table: &[(&str, T)], input: &mut Input<'_>) -> T {
    let name = input.next().expect("member name");
    table
        .iter()
        .find(|(entry, _)| *entry == name)
        .unwrap_or_else(|| panic!("unknown ascii_string_api member {name:?}"))
        .1
}
