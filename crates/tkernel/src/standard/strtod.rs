//! Locale-independent numeric-prefix conversion; decimal rounding uses Rust's parser.
mod hexadecimal;
mod special;

/// Parses a C-locale numeric prefix and returns its byte end offset; `strtod(" 1.5tail") == (1.5,4)`.
/// No conversion returns `(0.0, 0)`; embedded NUL terminates input as in C.
pub fn strtod(input: &str) -> (f64, usize) {
    strtod_bytes(input.as_bytes())
}

/// Parses the numeric prefix of a NUL-terminated byte string, including arbitrary suffix bytes;
/// `strtod_bytes(b" 1.5\xfftail") == (1.5, 4)`.
///
/// Native `Strtod(const char*, char**)` accepts byte strings that cannot be represented by `&str`.
pub fn strtod_bytes(input: &[u8]) -> (f64, usize) {
    let bytes = c_prefix(input);
    parse_prefix(bytes, leading_space_end(bytes))
}

fn c_prefix(input: &[u8]) -> &[u8] {
    let end = input
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(input.len());
    &input[..end]
}

fn leading_space_end(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|byte| !matches!(*byte, b' ' | b'\t' | b'\n' | b'\r' | 11 | 12))
        .unwrap_or(bytes.len())
}

fn parse_prefix(bytes: &[u8], start: usize) -> (f64, usize) {
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
    parse_decimal_prefix(bytes, start, digits)
}

fn parse_decimal_prefix(bytes: &[u8], start: usize, digits: usize) -> (f64, usize) {
    let end = decimal_end(bytes, digits);
    if end == digits {
        return (0.0, 0);
    }
    let token = std::str::from_utf8(&bytes[start..end]).expect("validated ASCII decimal prefix");
    (token.parse::<f64>().expect("validated decimal prefix"), end)
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
