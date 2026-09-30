//! The SJIS/EUC/GB and ANSI loops of Resource_Unicode.cxx, shared by their table functions.
use super::convert_unicode::{self, BytePair};
use crate::{standard::Failure, tcollection::ExtendedString};

/// The C string of a `const char*` argument: the bytes before the first NUL.
pub(super) fn c_bytes(text: &[u8]) -> &[u8] {
    &text[..text
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(text.len())]
}
/// One lead-byte decoder: bytes accepted as a lead, and the pair converter.
pub(super) struct PairDecoder {
    pub(super) is_lead: fn(u32) -> bool,
    pub(super) convert: fn(BytePair) -> BytePair,
}
/// The loop of ConvertSJIS/EUC/GBToUnicode: a lead byte takes the next byte as its pair,
/// any other byte is widened; a zero result is dropped, as `AssignCat` of an empty string.
///
/// A lead byte directly before the terminator makes the native loop read the NUL as the
/// trail and then run past the string (undefined behavior); here the NUL trail ends the text.
pub(super) fn decode_pairs(from: &[u8], decoder: &PairDecoder) -> ExtendedString {
    let bytes = c_bytes(from);
    let mut units = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = u32::from(bytes[index]);
        if !(decoder.is_lead)(byte) {
            units.push(byte as u16);
            index += 1;
            continue;
        }
        let trail = bytes.get(index + 1).map_or(0, |&b| u32::from(b));
        let (ph, pl) = (decoder.convert)((byte, trail));
        let unit = ((ph << 8) | pl) as u16;
        if unit != 0 {
            units.push(unit);
        }
        index += 2;
    }
    ExtendedString::from_raw(&units)
}
/// One pair encoder of ConvertUnicodeToSJIS/EUC/GB.
pub(super) struct PairEncoder {
    pub(super) convert: fn(BytePair) -> BytePair,
    pub(super) is_double: fn(u32) -> bool,
    /// Offset of the terminator written when a pair no longer fits: 0 for SJIS, -1 for
    /// EUC and GB (so EUC/GB overwrite the last single byte already written).
    pub(super) truncation_offset: i32,
    /// GB writes the source unit's low byte for a single byte, the others the converted one.
    pub(super) single_from_source: bool,
}
/// The buffer store of the native `tostr[index] = byte`.
///
/// Rust deviation for native undefined behavior: an index outside `0..to.len()` (EUC/GB
/// truncating before anything was written, a buffer of size 0, or UTF-8 into a buffer
/// exactly the encoded size) raises Standard_OutOfRange instead of writing out of bounds.
pub(super) fn put(to: &mut [u8], index: i32, byte: u8) -> Result<(), Failure> {
    let length = to.len();
    let slot = usize::try_from(index).ok().and_then(|i| to.get_mut(i));
    let Some(slot) = slot else {
        return Err(Failure::out_of_range(&format!(
            "Resource_Unicode buffer index {index}; expected 0..{length}"
        )));
    };
    *slot = byte;
    Ok(())
}
/// The ConvertUnicodeToSJIS/EUC/GB loop over a buffer of `maxsize = to.len()` bytes.
pub(super) fn encode_pairs(
    from: &ExtendedString,
    to: &mut [u8],
    encoder: &PairEncoder,
) -> Result<bool, Failure> {
    let max_size = to.len() as i32;
    let mut written: i32 = 0;
    for &unit in from.as_utf16() {
        if !encode_unit(unit, to, &mut written, encoder)? {
            return Ok(false);
        }
        if written >= max_size - 1 {
            put(to, max_size - 1, 0)?;
            return Ok(false);
        }
    }
    put(to, written, 0)?;
    Ok(true)
}
/// Stores one unit; false when a pair no longer fits (after the truncation terminator).
fn encode_unit(
    unit: u16,
    to: &mut [u8],
    written: &mut i32,
    encoder: &PairEncoder,
) -> Result<bool, Failure> {
    let (ph, pl) = (encoder.convert)((u32::from(unit >> 8), u32::from(unit & 0xFF)));
    if !(encoder.is_double)(ph) {
        let single = if encoder.single_from_source {
            unit as u8
        } else {
            pl as u8
        };
        put(to, *written, single)?;
        *written += 1;
        return Ok(true);
    }
    if *written >= to.len() as i32 - 3 {
        put(to, *written + encoder.truncation_offset, 0)?;
        return Ok(false);
    }
    put(to, *written, ph as u8)?;
    put(to, *written + 1, pl as u8)?;
    *written += 2;
    Ok(true)
}
/// Native ConvertUnicodeToANSI: units above 0xFF become spaces.
///
/// Quirk preserved: the size test also runs after the terminator is written, so a text
/// that exactly fills `maxsize - 1` bytes (and an empty text with `maxsize == 1`) reports
/// false although it was converted completely.
pub(super) fn encode_ansi(from: &ExtendedString, to: &mut [u8]) -> Result<bool, Failure> {
    let max_size = to.len() as i32;
    let units = from.as_utf16();
    let mut written: i32 = 0;
    let mut next = 0;
    loop {
        let finished = next >= units.len();
        if finished {
            put(to, written, 0)?;
        } else {
            let unit = units[next];
            next += 1;
            put(to, written, if unit >> 8 == 0 { unit as u8 } else { b' ' })?;
            written += 1;
        }
        if written >= max_size - 1 {
            put(to, max_size - 1, 0)?;
            return Ok(false);
        }
        if finished {
            return Ok(true);
        }
    }
}
/// Shift-JIS: leads 0x81..=0x9F and 0xE0..=0xEF.
pub(super) const SJIS_DECODER: PairDecoder = PairDecoder {
    is_lead: convert_unicode::is_sjis1,
    convert: convert_unicode::sjis_to_unicode,
};
/// EUC: leads 0xA1..=0xFE.
pub(super) const EUC_DECODER: PairDecoder = PairDecoder {
    is_lead: convert_unicode::is_euc,
    convert: convert_unicode::euc_to_unicode,
};
/// GB2312: every byte from 0x80 leads.
pub(super) const GB_DECODER: PairDecoder = PairDecoder {
    is_lead: convert_unicode::is_shift,
    convert: convert_unicode::gb_to_unicode,
};
pub(super) const SJIS_ENCODER: PairEncoder = PairEncoder {
    convert: convert_unicode::unicode_to_sjis,
    is_double: convert_unicode::is_sjis1,
    truncation_offset: 0,
    single_from_source: false,
};
pub(super) const EUC_ENCODER: PairEncoder = PairEncoder {
    convert: convert_unicode::unicode_to_euc,
    is_double: convert_unicode::is_euc,
    truncation_offset: -1,
    single_from_source: false,
};
pub(super) const GB_ENCODER: PairEncoder = PairEncoder {
    convert: convert_unicode::unicode_to_gb,
    is_double: convert_unicode::is_shift,
    truncation_offset: -1,
    single_from_source: true,
};
