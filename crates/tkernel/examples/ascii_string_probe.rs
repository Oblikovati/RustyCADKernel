//! Exact byte-string formatting, hashing and numeric-prefix oracle probe.
use rustycad_tkernel::tcollection::AsciiString;
use std::io::{self, BufRead};
fn decode(hex: &str) -> Vec<u8> {
    if hex == "-" {
        return Vec::new();
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn inspect(value: &AsciiString) {
    let integer = value
        .integer_value()
        .map(|v| v.to_string())
        .unwrap_or("x".into());
    let real = value
        .real_value()
        .map(|v| format!("{:016x}", v.to_bits()))
        .unwrap_or("x".into());
    println!(
        "{} {} {} {} {} {integer} {real}",
        value.len(),
        value.useful_length(),
        u8::from(value.is_integer_value()),
        u8::from(value.is_real_value(false)),
        u8::from(value.is_real_value(true))
    );
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let (mode, encoded) = line.split_once(' ').unwrap();
        if mode == "f" {
            let real = f64::from_bits(u64::from_str_radix(encoded, 16).unwrap());
            let formatted = AsciiString::from_real(real);
            println!(
                "{}",
                formatted
                    .as_bytes()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>()
            );
            continue;
        }
        let bytes = decode(encoded);
        let value = if mode == "r" {
            AsciiString::from_raw(&bytes)
        } else {
            AsciiString::from_view(&bytes)
        };
        if mode == "h" {
            println!("{:x}", value.hash_code());
        } else {
            inspect(&value);
        }
    }
}
