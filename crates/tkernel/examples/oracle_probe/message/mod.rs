//! Message differential operations (twin of `tools/oracle/tkernel_message_probe.cxx`).
//!
//! Strings cross the protocol as hex: bytes as pairs (`-` when empty), UTF-16 text as
//! comma-separated units (`-` when empty). A failure inside a step prints ` E<type>`.
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::str::SplitWhitespace;

mod output;
mod report;
mod text;

/// Dispatch `op` to the family that owns it; `None` when no family does.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    text::evaluate(op, input)
        .or_else(|| report::evaluate(op, input))
        .or_else(|| output::evaluate(op, input))
}

/// Byte pairs, `-` for none (the native `msgp_hex`).
fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".to_owned();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Bytes of a hex token (the native `msgp_bytes`).
fn bytes(token: &str) -> Vec<u8> {
    if token == "-" {
        return Vec::new();
    }
    (0..token.len() / 2)
        .map(|index| u8::from_str_radix(&token[2 * index..2 * index + 2], 16).expect("hex byte"))
        .collect()
}

/// The stored bytes of an ASCII string (the native `msgp_ascii`).
fn ascii(text: &AsciiString) -> String {
    hex(text.as_bytes())
}

/// Comma-separated hex units, `-` for none (the native `msgp_units`).
fn units(text: &ExtendedString) -> String {
    if text.is_empty() {
        return "-".to_owned();
    }
    let parts: Vec<String> = text
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:x}"))
        .collect();
    parts.join(",")
}

/// Extended text of a units token (the native `msgp_ext`).
fn ext(token: &str) -> ExtendedString {
    if token == "-" {
        return ExtendedString::default();
    }
    let units: Vec<u16> = token
        .split(',')
        .map(|unit| u16::from_str_radix(unit, 16).expect("hex unit"))
        .collect();
    ExtendedString::from_utf16(&units)
}

/// A double from its hex bit pattern.
fn real(bits: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(bits, 16).expect("hex bits"))
}

/// The remaining tokens of a request.
fn rest(input: &mut SplitWhitespace<'_>) -> Vec<String> {
    input.map(str::to_owned).collect()
}

/// `true`/`false` as the native `ostream << bool`.
fn bit(value: bool) -> u8 {
    u8::from(value)
}
