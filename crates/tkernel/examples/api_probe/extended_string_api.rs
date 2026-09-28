//! Rust side of the differential cases for TCollection_ExtendedString and TCollection_HExtendedString.
//!
//! Field and response formats follow tools/oracle/tkernel_extended_string_api_probe.cxx:
//! code-unit strings are hex of little-endian UTF-16 units, "null" is a null pointer, a
//! string prints as "<Length> <units hex>" and a failure as "ERR <class>" only, because the
//! Rust messages add the offending value to OCCT's text.
use super::common::{Case, decode, hex, next, next_bytes};
use rustycad_tkernel::{standard::Failure, tcollection::ExtendedString};
use std::str::SplitWhitespace;
mod edit;
mod handle;
mod string;

/// Operation name and handler of every case this area answers.
pub const CASES: &[(&str, Case)] = &[
    ("extended_string_new", string::new_case),
    ("extended_string_convert", string::convert_case),
    ("extended_string_numeric", string::numeric_case),
    ("extended_string_append", edit::append_case),
    ("extended_string_edit", edit::edit_case),
    ("extended_string_span", string::span_case),
    ("extended_string_whole", string::whole_case),
    ("extended_string_token", string::token_case),
    ("extended_string_locate", string::locate_case),
    ("extended_string_value", string::value_case),
    ("extended_string_hnew", handle::hnew_case),
    ("extended_string_hedit", handle::hedit_case),
    ("extended_string_hkey", handle::hkey_case),
];

/// A named sub-operation of one case.
type SubCase = fn(&mut SplitWhitespace<'_>) -> String;

/// Next field decoded as little-endian UTF-16 units; `next_units(&mut input)`.
fn next_units(input: &mut SplitWhitespace<'_>) -> Vec<u16> {
    units_of(&next_bytes(input))
}

fn units_of(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect()
}

/// Next field as a nullable pointer span: `None` for "null".
fn next_span(input: &mut SplitWhitespace<'_>) -> Option<Vec<u16>> {
    let token = input.next().expect("span field");
    (token != "null").then(|| units_of(&decode(token)))
}

/// Next field as an explicit-length string (the `std::u16string_view` constructor).
fn next_string(input: &mut SplitWhitespace<'_>) -> ExtendedString {
    ExtendedString::from_raw(&next_units(input))
}

/// Next field as the IEEE-754 bit pattern of a real.
fn next_real(input: &mut SplitWhitespace<'_>) -> f64 {
    f64::from_bits(next::<u64>(input))
}

/// The units a `const char16_t*` names: everything before the first zero.
fn c_units(units: &[u16]) -> &[u16] {
    let end = units.iter().position(|&unit| unit == 0);
    &units[..end.unwrap_or(units.len())]
}

fn units_hex(units: &[u16]) -> String {
    hex(&units
        .iter()
        .flat_map(|unit| unit.to_le_bytes())
        .collect::<Vec<_>>())
}

/// "<Length> <units hex>", as the native `describe`.
fn describe(string: &ExtendedString) -> String {
    format!("{} {}", string.length(), units_hex(string.as_utf16()))
}

fn flag(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

/// "OK <text>" or "ERR <class>", as the native `guarded_kind`.
fn guarded_kind(result: Result<String, Failure>) -> String {
    match result {
        Ok(text) => format!("OK {text}"),
        Err(failure) => format!("ERR {}", failure.exception_type()),
    }
}

/// Runs the sub-operation named by the next field.
fn dispatch(table: &[(&str, SubCase)], input: &mut SplitWhitespace<'_>) -> String {
    let name = input.next().expect("sub-operation");
    let case = table
        .iter()
        .find(|(entry, _)| *entry == name)
        .unwrap_or_else(|| panic!("unknown extended-string sub-operation {name:?}"));
    (case.1)(input)
}
