//! Resource package probe operations (twin of `tools/oracle/tkernel_resource_probe.cxx`).
use rustycad_tkernel::{
    resource::{FormatType, ResourceUnicode},
    standard::Failure,
    tcollection::ExtendedString,
};
use std::str::SplitWhitespace;

mod manager;

/// The output of a Resource operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "res_decode" => decode(input),
        "res_gbk" | "res_big5" => direct(op, input),
        "res_encode" => encode(input),
        "res_path" => manager::path(input),
        "res_load" => manager::load(input),
        "res_dirs" => manager::directories(input),
        "res_save" => manager::save(input),
        "res_ext" => manager::ext(input),
        "res_charset" => manager::charset(input),
        _ => return None,
    })
}
/// Lowercase hex, `-` for empty (native `res_hex`).
pub fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
/// Bytes of a hex token, `-` for empty (native `res_unhex`).
pub fn unhex(text: &str) -> Vec<u8> {
    if text == "-" {
        return Vec::new();
    }
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).expect("hex byte"))
        .collect()
}
pub fn token<'a>(input: &mut SplitWhitespace<'a>) -> &'a str {
    input.next().expect("resource probe field")
}
fn format(input: &mut SplitWhitespace<'_>) -> FormatType {
    FormatType::from_raw(token(input).parse().expect("format")).expect("known format")
}
/// `n=<length> u=<4-hex units>` (native `res_units`).
pub fn units(text: &ExtendedString) -> String {
    let hex: String = text.as_utf16().iter().map(|u| format!("{u:04x}")).collect();
    format!(
        "n={} u={}",
        text.len(),
        if hex.is_empty() { "-" } else { &hex }
    )
}
/// The UTF-16 units of a hex token, embedded zeros kept (native `res_ext_string`).
pub fn ext_string(text: &str) -> ExtendedString {
    if text == "-" {
        return ExtendedString::default();
    }
    let units: Vec<u16> = (0..text.len() / 4)
        .map(|i| u16::from_str_radix(&text[4 * i..4 * i + 4], 16).expect("hex unit"))
        .collect();
    ExtendedString::from_raw(&units)
}
pub fn failure(error: &Failure) -> String {
    format!("throw {}: {} ", error.exception_type(), error.message())
}
fn decode(input: &mut SplitWhitespace<'_>) -> String {
    let format = format(input);
    let source = unhex(token(input));
    let mut text = ExtendedString::default();
    let prefix = ResourceUnicode::convert_format_to_unicode(format, &source, &mut text)
        .err()
        .map_or_else(String::new, |error| failure(&error));
    prefix + &units(&text)
}
fn direct(op: &str, input: &mut SplitWhitespace<'_>) -> String {
    let source = unhex(token(input));
    let mut text = ExtendedString::default();
    let result = if op == "res_gbk" {
        ResourceUnicode::convert_gbk_to_unicode(&source, &mut text)
    } else {
        ResourceUnicode::convert_big5_to_unicode(&source, &mut text)
    };
    let prefix = match result {
        Ok(done) => format!("b={} ", u8::from(done)),
        Err(error) => failure(&error),
    };
    prefix + &units(&text)
}
fn encode(input: &mut SplitWhitespace<'_>) -> String {
    let format = format(input);
    let max_size: usize = token(input).parse().expect("maxsize");
    let text = ext_string(token(input));
    let mut buffer = vec![0xA5u8; max_size];
    let prefix = match ResourceUnicode::convert_unicode_to_format(format, &text, &mut buffer) {
        Ok(done) => format!("b={} ", u8::from(done)),
        Err(error) => failure(&error),
    };
    format!("{prefix}buf={}", hex(&buffer))
}
