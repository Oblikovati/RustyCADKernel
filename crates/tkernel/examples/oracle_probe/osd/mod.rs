//! OSD differential operations, one module per class family.
//!
//! Each family answers the operations it owns with the text the native twin in
//! `tools/oracle/tkernel_osd_<family>_probe.cxx` prints, or `None` for an operation it
//! does not own.
use std::str::SplitWhitespace;

mod file_system;
mod files;
mod foundation;
mod path;
mod process;
mod threads;
mod timing;

/// Dispatch `op` to the family that owns it; `None` when no family does.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    foundation::evaluate(op, input)
        .or_else(|| path::evaluate(op, input))
        .or_else(|| files::evaluate(op, input))
        .or_else(|| file_system::evaluate(op, input))
        .or_else(|| process::evaluate(op, input))
        .or_else(|| timing::evaluate(op, input))
        .or_else(|| threads::evaluate(op, input))
}

/// Reads one request token, `-` standing for the empty string (native `osd_token`).
#[allow(dead_code)] // Shared by the families; not every family reads text tokens.
pub fn token(input: &mut SplitWhitespace<'_>) -> String {
    let token: &str = input.next().expect("text token");
    if token == "-" {
        String::new()
    } else {
        token.to_string()
    }
}

/// Reads one integer token.
#[allow(dead_code)] // Shared by the families.
pub fn integer(input: &mut SplitWhitespace<'_>) -> i64 {
    input
        .next()
        .expect("integer token")
        .parse()
        .expect("integer input")
}

/// Brackets text on one line with `\\`, newline, carriage return and tab escaped
/// (native `osd_escape`).
#[allow(dead_code)] // Shared by the families.
pub fn escape(text: &str) -> String {
    let mut out: String = String::from("[");
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push(']');
    out
}

/// Prints a boolean as the native `ostream << bool` does.
#[allow(dead_code)] // Shared by the families.
pub fn bit(value: bool) -> u8 {
    u8::from(value)
}
