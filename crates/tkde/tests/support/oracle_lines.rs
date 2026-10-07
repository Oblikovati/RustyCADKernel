//! The line format of the native oracle: escapes, booleans as 0/1, reals as `%g`.
use rustycad_tkernel::{ncollection::List, tcollection::AsciiString};

/// The native `esc`: new lines, tabs, carriage returns and `|` made visible.
pub fn esc(text: &AsciiString) -> String {
    let mut escaped = String::new();
    for &byte in text.as_bytes() {
        match byte {
            b'\n' => escaped.push_str("\\n"),
            b'\t' => escaped.push_str("\\t"),
            b'\r' => escaped.push_str("\\r"),
            b'|' => escaped.push_str("\\p"),
            _ => escaped.push(char::from(byte)),
        }
    }
    escaped
}

pub fn s(text: &str) -> AsciiString {
    AsciiString::new(text)
}

pub fn b(value: bool) -> u8 {
    u8::from(value)
}

pub fn real(value: f64) -> String {
    esc(&AsciiString::from_real(value))
}

/// The collected `key|value` lines.
#[derive(Default)]
pub struct Lines(pub Vec<String>);
impl Lines {
    pub fn line(&mut self, key: &str, value: impl std::fmt::Display) {
        self.0.push(format!("{key}|{value}"));
    }
}
pub fn list(items: &[&str]) -> List<AsciiString> {
    let mut result: List<AsciiString> = List::default();
    for item in items {
        result.append(s(item));
    }
    result
}
