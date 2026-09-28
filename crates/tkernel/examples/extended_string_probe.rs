//! Exhaustive UTF-16 hash, classification, and byte conversion oracle probe.
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::io::{self, BufRead};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn probe(encoded: &str) {
    let units: Vec<u16> = if encoded == "-" {
        Vec::new()
    } else {
        (0..encoded.len())
            .step_by(4)
            .map(|i| u16::from_str_radix(&encoded[i..i + 4], 16).unwrap())
            .collect()
    };
    let value = ExtendedString::from_raw(&units);
    let size = value.utf8_length();
    let mut buffer = vec![0; size + 1];
    let written = value.write_utf8(&mut buffer).unwrap();
    let ascii = AsciiString::from_extended(&value, None);
    let replaced = AsciiString::from_extended(&value, Some(b'?'));
    println!(
        "{} {} {:x} {size} {written} {} {} {}",
        value.len(),
        u8::from(value.is_ascii()),
        value.hash_code(),
        hex(&buffer[..written]),
        hex(ascii.as_bytes()),
        hex(replaced.as_bytes())
    );
}
fn main() {
    for line in io::stdin().lock().lines() {
        probe(&line.unwrap());
    }
}
