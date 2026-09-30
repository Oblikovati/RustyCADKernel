//! `UsefullLength` over OCCT's permissive `NCollection_UtfIterator<char>` decoding.
//!
//! The iterator takes the sequence length from the lead byte alone and never validates
//! continuation bytes, so it may step over an embedded NUL, and a sequence can decode to
//! 0 (`C0 80`) and end the scan early. Both behaviors are reproduced byte for byte.
use super::AsciiString;

/// Values subtracted after accumulation (`offsetsFromUTF8`).
const UTF8_OFFSETS: [u32; 6] = [
    0x0000_0000,
    0x0000_3080,
    0x000E_2080,
    0x03C8_2080,
    0xFA08_2080,
    0x8208_2080,
];

impl AsciiString {
    /// `UsefullLength`: byte end of the last code point that is non-ASCII or an ASCII graphic
    /// character, scanning up to the first decoded zero; `AsciiString::new("ab  ").useful_length() == 2`.
    ///
    /// A sequence whose announced continuation bytes run past the terminator makes OCCT read
    /// its allocation padding; the scan stops there instead.
    pub fn useful_length(&self) -> i32 {
        let bytes = self.as_bytes_with_nul();
        let (mut position, mut graphic_end) = (0, 0);
        while let Some((code, next)) = read_utf8(bytes, position) {
            if code == 0 {
                break;
            }
            if code > 0x7F || (code as u8).is_ascii_graphic() {
                graphic_end = next;
            }
            position = next;
        }
        graphic_end as i32
    }
}

/// Continuation-byte count for a lead byte: 1 for C0..DF, 2 for E0..EF, 3 for F0..F7,
/// 4 for F8..FB and 5 for FC..FF, 0 otherwise.
fn trailing_bytes(lead: u8) -> usize {
    match lead {
        0xC0..=0xDF => 1,
        0xE0..=0xEF => 2,
        0xF0..=0xF7 => 3,
        0xF8..=0xFB => 4,
        0xFC..=0xFF => 5,
        _ => 0,
    }
}

/// `readUTF8` at `position`: the wrapping `char32_t` value and the next position, or `None`
/// when the sequence would read past the buffer.
fn read_utf8(bytes: &[u8], position: usize) -> Option<(u32, usize)> {
    let trailing = trailing_bytes(*bytes.get(position)?);
    let sequence = bytes.get(position..=position + trailing)?;
    let accumulated = sequence.iter().fold(0_u32, |code, &byte| {
        code.wrapping_shl(6).wrapping_add(u32::from(byte))
    });
    Some((
        accumulated.wrapping_sub(UTF8_OFFSETS[trailing]),
        position + trailing + 1,
    ))
}
