// Portions of code are copyrighted by Unicode, Inc. (c) 2001-2004; see
// NCollection_UtfIterator.lxx for the notice that must remain attached.

//! Code-unit readers and writers of NCollection_UtfIterator.lxx, operation for operation.

pub(super) const UTF16_SURROGATE_HIGH_START: u32 = 0xD800;
pub(super) const UTF16_SURROGATE_HIGH_END: u32 = 0xDBFF;
pub(super) const UTF16_SURROGATE_LOW_START: u32 = 0xDC00;
pub(super) const UTF16_SURROGATE_LOW_END: u32 = 0xDFFF;
const UTF16_SURROGATE_HIGH_SHIFT: u32 = 10;
const UTF16_SURROGATE_LOW_BASE: u32 = 0x0001_0000;
const UTF16_SURROGATE_LOW_MASK: u32 = 0x3FF;
const UTF32_MAX_BMP: u32 = 0x0000_FFFF;
pub(super) const UTF32_MAX_LEGAL: u32 = 0x0010_FFFF;
const UTF8_BYTE_MASK: u32 = 0xBF;
const UTF8_BYTE_MARK: u32 = 0x80;
const UTF8_FIRST_BYTE_MARK: [u32; 7] = [0x00, 0x00, 0xC0, 0xE0, 0xF0, 0xF8, 0xFC];
/// Magic values subtracted after accumulating a UTF-8 sequence (one per trailing-byte count).
const OFFSETS_FROM_UTF8: [u32; 6] = [
    0x0000_0000,
    0x0000_3080,
    0x000E_2080,
    0x03C8_2080,
    0xFA08_2080,
    0x8208_2080,
];

/// The code unit at `index`, or 0 past the end: slices model NUL-terminated buffers whose
/// trailing storage reads as zeros, so an unprotected overrun (native UB) stays defined.
pub(super) fn unit_at<T: super::UtfUnit>(buffer: &[T], index: usize) -> u32 {
    buffer.get(index).map_or(0, |&unit| unit.to_bits())
}

/// Trailing bytes announced by a UTF-8 lead byte (the native UTF8_BYTES_MINUS_ONE table).
fn utf8_trailing_bytes(lead: u32) -> usize {
    match lead {
        0x00..=0xBF => 0,
        0xC0..=0xDF => 1,
        0xE0..=0xEF => 2,
        0xF0..=0xF7 => 3,
        0xF8..=0xFB => 4,
        _ => 5,
    }
}

/// readUTF8: decodes without validation (5- and 6-byte forms included); returns (symbol, next).
pub(super) fn read_utf8<T: super::UtfUnit>(buffer: &[T], position: usize) -> (u32, usize) {
    let trailing = utf8_trailing_bytes(unit_at(buffer, position));
    let mut symbol: u32 = 0;
    // The native switch falls through: add, then shift, for every byte but the last.
    for offset in 0..trailing {
        symbol = symbol.wrapping_add(unit_at(buffer, position + offset));
        symbol <<= 6;
    }
    symbol = symbol.wrapping_add(unit_at(buffer, position + trailing));
    (
        symbol.wrapping_sub(OFFSETS_FROM_UTF8[trailing]),
        position + trailing + 1,
    )
}

/// readUTF16: joins a high surrogate with a following low one; a lone half stays as is.
pub(super) fn read_utf16(buffer: &[u16], position: usize) -> (u32, usize) {
    let first = unit_at(buffer, position);
    let is_high = (UTF16_SURROGATE_HIGH_START..=UTF16_SURROGATE_HIGH_END).contains(&first);
    let second = unit_at(buffer, position + 1);
    if !is_high || !(UTF16_SURROGATE_LOW_START..=UTF16_SURROGATE_LOW_END).contains(&second) {
        return (first, position + 1);
    }
    let symbol = ((first - UTF16_SURROGATE_HIGH_START) << UTF16_SURROGATE_HIGH_SHIFT)
        + (second - UTF16_SURROGATE_LOW_START)
        + UTF16_SURROGATE_LOW_BASE;
    (symbol, position + 2)
}

/// True for the surrogate range that UTF-32 forbids.
fn is_surrogate(symbol: u32) -> bool {
    (UTF16_SURROGATE_HIGH_START..=UTF16_SURROGATE_LOW_END).contains(&symbol)
}

/// AdvanceBytesUtf8: 0 for surrogates and values above U+10FFFF, else 1..=4.
pub(super) fn utf8_length(symbol: u32) -> usize {
    match symbol {
        _ if is_surrogate(symbol) => 0,
        0..=0x7F => 1,
        0x80..=0x7FF => 2,
        0x800..=0xFFFF => 3,
        0x1_0000..=UTF32_MAX_LEGAL => 4,
        _ => 0,
    }
}

/// AdvanceCodeUnitsUtf16: 0 for surrogates and illegal values, 2 above the BMP.
pub(super) fn utf16_length(symbol: u32) -> usize {
    match symbol {
        _ if is_surrogate(symbol) => 0,
        0..=UTF32_MAX_BMP => 1,
        0x1_0000..=UTF32_MAX_LEGAL => 2,
        _ => 0,
    }
}

/// GetUtf8 into `out` (sized by `utf8_length`); the continuation bytes are filled last-first.
pub(super) fn write_utf8<T: super::UtfUnit>(symbol: u32, out: &mut [T]) {
    let length = out.len();
    let mut rest = symbol;
    for slot in (1..length).rev() {
        out[slot] = T::from_bits((rest | UTF8_BYTE_MARK) & UTF8_BYTE_MASK);
        rest >>= 6;
    }
    if length > 0 {
        out[0] = T::from_bits(rest | UTF8_FIRST_BYTE_MARK[length]);
    }
}

/// GetUtf16 into `out` (sized by `utf16_length`): one unit, or a surrogate pair.
pub(super) fn write_utf16(symbol: u32, out: &mut [u16]) {
    if out.len() == 1 {
        out[0] = symbol as u16;
    } else if out.len() == 2 {
        let offset = symbol - UTF16_SURROGATE_LOW_BASE;
        out[0] = ((offset >> UTF16_SURROGATE_HIGH_SHIFT) + UTF16_SURROGATE_HIGH_START) as u16;
        out[1] = ((offset & UTF16_SURROGATE_LOW_MASK) + UTF16_SURROGATE_LOW_START) as u16;
    }
}
