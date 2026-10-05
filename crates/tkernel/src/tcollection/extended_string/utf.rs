//! The `NCollection_UtfIterator` conversions that `TCollection_ExtendedString` uses.
//!
//! OCCT's UTF-8 reader is "not protected against invalid UTF-8": it trusts the lead byte,
//! sums the following bytes whatever they are and subtracts the Unicode offset table in
//! unsigned 32-bit arithmetic. The result is only rejected when it exceeds U+10FFFF, so a
//! lone continuation byte or an overlong form decodes to some code point, and a decoded
//! zero (the overlong `C0 80`) ends the string. Writing UTF-16 or UTF-8 drops a value in
//! the surrogate range or above U+10FFFF, which is exactly the set `char::from_u32` rejects.

/// Offsets that the reader subtracts, indexed by the number of trailing bytes.
const OFFSETS_FROM_UTF8: [u32; 6] = [
    0x0000_0000,
    0x0000_3080,
    0x000E_2080,
    0x03C8_2080,
    0xFA08_2080,
    0x8208_2080,
];
/// `NCollection_UtfIterator::UTF32_MAX_LEGAL`.
const UTF32_MAX_LEGAL: u32 = 0x0010_FFFF;

/// Trailing bytes a lead byte announces (`UTF8_BYTES_MINUS_ONE`), including the illegal
/// five- and six-byte forms.
fn trailing_bytes(lead: u8) -> usize {
    match lead {
        0x00..=0xBF => 0,
        0xC0..=0xDF => 1,
        0xE0..=0xEF => 2,
        0xF0..=0xF7 => 3,
        0xF8..=0xFB => 4,
        0xFC..=0xFF => 5,
    }
}

/// Reads one symbol at `at` like `readUTF8`, returning the value and the next position, or
/// `None` when the announced bytes run past the end of the C string.
fn read_utf8(bytes: &[u8], at: usize) -> Option<(u32, usize)> {
    let next = at + trailing_bytes(bytes[at]) + 1;
    let sequence = bytes.get(at..next)?;
    let sum = sequence.iter().fold(0_u32, |value, &byte| {
        (value << 6).wrapping_add(u32::from(byte))
    });
    Some((sum.wrapping_sub(OFFSETS_FROM_UTF8[next - at - 1]), next))
}

/// Appends `value` as UTF-16 like `GetUtf16`, dropping surrogates and values above U+10FFFF.
fn push_utf16(value: u32, units: &mut Vec<u16>) {
    if let Some(symbol) = char::from_u32(value) {
        units.extend_from_slice(symbol.encode_utf16(&mut [0; 2]));
    }
}

/// Converts the bytes of a C string (no NUL inside) like `ConvertToUnicode`; `None` means
/// "not UTF-8": a symbol above U+10FFFF, or a sequence cut by the terminator, which OCCT
/// would read past (undefined behavior that the port treats as invalid input).
pub(super) fn utf8_to_utf16(bytes: &[u8]) -> Option<Vec<u16>> {
    let mut units = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let (value, next) = read_utf8(bytes, at)?;
        if value == 0 {
            break;
        }
        if value > UTF32_MAX_LEGAL {
            return None;
        }
        push_utf16(value, &mut units);
        at = next;
    }
    Some(units)
}

/// Converts UTF-32 code units up to the first zero like `fromWideString` on a 4-byte
/// `wchar_t`; `utf32_to_utf16(&[0x1F600, 0])`.
pub(super) fn utf32_to_utf16(values: &[u32]) -> Vec<u16> {
    let mut units = Vec::with_capacity(values.len());
    for &value in values.iter().take_while(|&&value| value != 0) {
        push_utf16(value, &mut units);
    }
    units
}

/// Converts UTF-16 up to the first zero unit to UTF-8 like `ToUTF8CString`: pairs combine,
/// unpaired surrogates are dropped; `utf16_to_utf8(&[0xD83D, 0xDE00])`.
pub(super) fn utf16_to_utf8(units: &[u16]) -> String {
    char::decode_utf16(c_units_prefix(units).iter().copied())
        .filter_map(Result::ok)
        .collect()
}

/// The part of a byte buffer that a C string reader sees: everything before the first NUL.
pub(super) fn c_string_prefix(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..end]
}

/// Returns the part read by an OCCT `const char16_t*` overload;
/// `c_units_prefix(&[65, 0, 66]) == &[65]`.
pub(super) fn c_units_prefix(units: &[u16]) -> &[u16] {
    let end = units
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(units.len());
    &units[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lenient_utf8_reader_matches_native_arithmetic() {
        assert_eq!(utf8_to_utf16(b"\xC3A"), Some(vec![0x81]));
        assert_eq!(utf8_to_utf16(b"\x80\xBF"), Some(vec![0x80, 0xBF]));
        assert_eq!(utf8_to_utf16(b"A\xC0\x80B"), Some(vec![0x41]));
        assert_eq!(utf8_to_utf16(b"\xED\xA0\x80Z"), Some(vec![0x5A]));
        assert_eq!(utf8_to_utf16(b"\xF4\x90\x80\x80"), None);
        assert_eq!(utf8_to_utf16(b"\xE2\x82"), None);
        assert_eq!(
            utf8_to_utf16("€😀".as_bytes()),
            Some(vec![0x20AC, 0xD83D, 0xDE00])
        );
    }
    #[test]
    fn wide_and_utf16_writers_drop_illegal_values() {
        assert_eq!(
            utf32_to_utf16(&[0x41, 0xD800, 0x11_0000, 0x1F600, 0, 0x42]),
            vec![0x41, 0xD83D, 0xDE00]
        );
        assert_eq!(utf16_to_utf8(&[0x41, 0xDC00, 0xD800, 0x42, 0, 0x43]), "AB");
        assert_eq!(c_string_prefix(b"ab\0cd"), b"ab");
    }
}
