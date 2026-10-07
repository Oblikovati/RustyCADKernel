//! The C string helpers LDOM relies on: a `char*` ends at its first NUL, and `strtol`
//! (C locale) reads the integers of attribute values and text nodes.

/// The C string of `bytes`: everything before the first NUL; `c_str(b"ab\0c") == b"ab"`.
pub(crate) fn c_str(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

/// The result of `strtol`: the value, the number of bytes consumed (0 without any digit,
/// the native `endptr == nptr`) and the `ERANGE` overflow flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Strtol {
    pub(crate) value: i64,
    pub(crate) consumed: usize,
    pub(crate) overflow: bool,
}

/// C `strtol(text, &end, base)` for base 10 or 16 in the C locale: leading white space, an
/// optional sign, an optional `0x` for base 16, then digits; a value beyond `long` clamps
/// and sets the overflow flag; `strtol(b" -12x", 10).value == -12`.
pub(crate) fn strtol(text: &[u8], base: u32) -> Strtol {
    let text = c_str(text);
    let mut index = text
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .count();
    let negative = text.get(index) == Some(&b'-');
    if matches!(text.get(index), Some(b'-' | b'+')) {
        index += 1;
    }
    if base == 16
        && text.get(index) == Some(&b'0')
        && matches!(text.get(index + 1), Some(b'x' | b'X'))
        && text
            .get(index + 2)
            .is_some_and(|byte| byte.is_ascii_hexdigit())
    {
        index += 2;
    }
    let digits = text[index..]
        .iter()
        .take_while(|byte| (**byte as char).is_digit(base))
        .count();
    if digits == 0 {
        return Strtol::default();
    }
    accumulate(&text[index..index + digits], base, negative, index + digits)
}

/// The digit loop of `strtol`, clamping to `i64` (the 64-bit `long`).
fn accumulate(digits: &[u8], base: u32, negative: bool, consumed: usize) -> Strtol {
    let mut magnitude: i128 = 0;
    for digit in digits {
        let value = (*digit as char).to_digit(base).unwrap_or(0);
        magnitude = (magnitude * i128::from(base) + i128::from(value)).min(i128::from(u64::MAX));
    }
    let signed = if negative { -magnitude } else { magnitude };
    let clamped = signed.clamp(i128::from(i64::MIN), i128::from(i64::MAX));
    Strtol {
        value: clamped as i64,
        consumed,
        overflow: clamped != signed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strtol_reads_like_c() {
        assert_eq!(
            strtol(b"  42abc", 10),
            Strtol {
                value: 42,
                consumed: 4,
                overflow: false
            }
        );
        assert_eq!(strtol(b"abc", 10).consumed, 0);
        assert_eq!(strtol(b"-0x1F", 16).value, -31);
        assert!(strtol(b"99999999999999999999", 10).overflow);
        assert_eq!(
            strtol(b"0x", 16),
            Strtol {
                value: 0,
                consumed: 1,
                overflow: false
            }
        );
        assert_eq!(c_str(b"ab\0cd"), b"ab");
    }
}
