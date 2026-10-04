//! C-locale fallback for non-Unix/non-Windows targets without a locale API adapter.
use crate::ncollection::WideChar;
/// The C-locale `mblen(text, MB_CUR_MAX)`: 0 for an empty string, 1 for ASCII, -1 otherwise.
fn c_locale_mblen(text: &[u8]) -> i32 {
    match text.first().copied().unwrap_or(0) {
        0 => 0,
        0x01..=0x7F => 1,
        _ => -1,
    }
}

/// The C-locale `wcstombs(buffer, wide, buffer.len())`: bytes stored before the terminator,
/// `usize::MAX` (the native `(size_t)-1`) on a non-ASCII character. A full buffer is left
/// without a terminator, as the C function does.
pub(super) fn encode(wide: &[WideChar], buffer: &mut [u8]) -> usize {
    for (written, slot) in buffer.iter_mut().enumerate() {
        #[allow(clippy::useless_conversion)] // WideChar is u16 on Windows
        let symbol: u32 = wide.get(written).map_or(0, |&unit| unit.into());
        if symbol > 0x7F {
            return usize::MAX;
        }
        *slot = symbol as u8;
        if symbol == 0 {
            return written;
        }
    }
    buffer.len()
}

pub(super) fn decode(text: &[u8]) -> Vec<WideChar> {
    if c_locale_mblen(text) <= 0 {
        return Vec::new();
    }
    vec![WideChar::from(text[0]), 0]
}
