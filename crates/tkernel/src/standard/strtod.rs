//! Locale-independent numeric-prefix conversion; decimal rounding uses Rust's parser.
mod hexadecimal;
mod special;

/// Parses a C-locale numeric prefix and returns its byte end offset; `strtod(" 1.5tail") == (1.5,4)`.
/// No conversion returns `(0.0, 0)`; embedded NUL terminates input as in C.
pub fn strtod(input: &str) -> (f64, usize) {
    let input = input.split('\0').next().unwrap_or_default();
    let bytes = input.as_bytes();
    let start = bytes
        .iter()
        .position(|b| !matches!(*b, b' ' | b'\t' | b'\n' | b'\r' | 11 | 12))
        .unwrap_or(bytes.len());
    let signed = matches!(bytes.get(start), Some(b'+') | Some(b'-'));
    let digits = start + usize::from(signed);
    let negative = bytes.get(start) == Some(&b'-');
    if let Some(result) = special::parse_special(bytes, digits, negative) {
        return result;
    }
    if bytes
        .get(digits..digits + 2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"0x"))
    {
        return hexadecimal::parse_hexadecimal(bytes, digits, negative);
    }
    let end = decimal_end(bytes, digits);
    if end == digits {
        return (0.0, 0);
    }
    let value = input[start..end]
        .parse::<f64>()
        .expect("validated decimal prefix");
    (value, end)
}
fn decimal_end(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    let before = cursor - start;
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
    }
    let fractional = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if before == 0 && cursor == fractional {
        return start;
    }
    exponent_end(bytes, cursor, b'e')
}
fn exponent_end(bytes: &[u8], start: usize, marker: u8) -> usize {
    if !bytes
        .get(start)
        .is_some_and(|b| b.to_ascii_lowercase() == marker)
    {
        return start;
    }
    let mut cursor = start + 1;
    if matches!(bytes.get(cursor), Some(b'+') | Some(b'-')) {
        cursor += 1;
    }
    let digits = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == digits { start } else { cursor }
}
