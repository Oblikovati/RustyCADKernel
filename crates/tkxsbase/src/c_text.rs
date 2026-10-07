//! The C library conversions the native packages call on texts (`atoi`).

/// `atoi(text)`: optional C spaces and sign, then decimal digits; 0 without digits.
/// Overflow is undefined natively; glibc's `strtol` saturates to `long`, then the value
/// is truncated to `int`, which the port reproduces; `atoi(b" -12x") == -12`.
pub(crate) fn atoi(text: &[u8]) -> i32 {
    let start = text
        .iter()
        .position(|b| !is_c_space(*b))
        .unwrap_or(text.len());
    let mut rest = &text[start..];
    let negative = rest.first() == Some(&b'-');
    if matches!(rest.first(), Some(b'+' | b'-')) {
        rest = &rest[1..];
    }
    let mut magnitude: i128 = 0;
    for digit in rest.iter().take_while(|b| b.is_ascii_digit()) {
        magnitude = (magnitude * 10 + i128::from(digit - b'0')).min(i128::from(u64::MAX));
    }
    let signed = if negative { -magnitude } else { magnitude };
    // Truncation to `int` after the `long` clamp is the glibc behaviour of `atoi`.
    signed.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64 as i32
}

fn is_c_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r')
}

#[cfg(test)]
mod tests {
    use super::atoi;

    #[test]
    fn c_atoi_prefixes() {
        assert_eq!(atoi(b" -12x"), -12);
        assert_eq!(atoi(b"+7"), 7);
        assert_eq!(atoi(b"abc"), 0);
        assert_eq!(atoi(b""), 0);
        assert_eq!(atoi(b"4294967297"), 1);
    }
}
