//! Request parsing and response formatting shared by every TKernel API probe area.
use rustycad_tkernel::standard::Failure;
use std::str::SplitWhitespace;

/// One probe operation: reads the rest of a request line, returns one response line.
pub type Case = fn(&mut SplitWhitespace<'_>) -> String;

/// Lower-case hex of raw bytes, "-" when empty, matching `tkernel_probe::hex`; `hex(b"A")`.
pub fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Inverse of [`hex`]; `decode("4100") == b"A\0"`.
pub fn decode(text: &str) -> Vec<u8> {
    if text == "-" {
        return Vec::new();
    }
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).expect("hex byte pair"))
        .collect()
}

/// Next request token decoded from hex; `next_bytes(&mut input)`.
pub fn next_bytes(input: &mut SplitWhitespace<'_>) -> Vec<u8> {
    decode(input.next().expect("hex field"))
}

#[allow(
    dead_code,
    reason = "shared by the areas; unused until one parses numbers"
)]
/// Next request token parsed as `T`; `next::<i32>(&mut input)`.
pub fn next<T: std::str::FromStr>(input: &mut SplitWhitespace<'_>) -> T {
    let token = input.next().expect("request field");
    token
        .parse()
        .unwrap_or_else(|_| panic!("request field {token:?} has the wrong shape"))
}

#[allow(
    dead_code,
    reason = "shared by the areas; unused until one reports errors"
)]
/// "OK <text>" or "ERR <type>: <message>", as `tkernel_probe::guarded` prints.
pub fn guarded(result: Result<String, Failure>) -> String {
    match result {
        Ok(text) => format!("OK {text}"),
        Err(failure) => format!("ERR {}: {}", failure.exception_type(), failure.message()),
    }
}
