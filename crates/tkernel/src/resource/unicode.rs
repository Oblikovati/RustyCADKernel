// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Resource_Unicode: conversions between byte encodings and UTF-16 extended strings.
use super::{
    FormatType, cjk, code_page,
    multibyte::{self, c_bytes},
};
use crate::{
    ncollection::{UtfIterator, UtfString},
    standard::Failure,
    tcollection::ExtendedString,
};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicI32},
};
mod format;

/// Native static class Resource_Unicode; every member is an associated function.
///
/// ```
/// use rustycad_tkernel::{resource::{FormatType, ResourceUnicode}, tcollection::ExtendedString};
/// let mut text = ExtendedString::default();
/// ResourceUnicode::convert_format_to_unicode(FormatType::Utf8, "é".as_bytes(), &mut text).unwrap();
/// assert_eq!(text.as_utf16(), &[0xE9]);
/// ```
pub struct ResourceUnicode;

/// Process-wide current format, as the native file statics `TheFormat`/`AlreadyRead`.
static FORMAT: AtomicI32 = AtomicI32::new(FormatType::ANSI as i32);
static FORMAT_READ: AtomicBool = AtomicBool::new(false);
static FORMAT_LOCK: Mutex<()> = Mutex::new(());

impl ResourceUnicode {
    /// Shift-JIS bytes (up to the first NUL) to UTF-16; `ResourceUnicode::convert_sjis_to_unicode(b"A", &mut text)`.
    pub fn convert_sjis_to_unicode(from: &[u8], to: &mut ExtendedString) {
        *to = multibyte::decode_pairs(from, &multibyte::SJIS_DECODER);
    }
    /// EUC bytes (Japanese tables) to UTF-16; `ResourceUnicode::convert_euc_to_unicode(b"A", &mut text)`.
    pub fn convert_euc_to_unicode(from: &[u8], to: &mut ExtendedString) {
        *to = multibyte::decode_pairs(from, &multibyte::EUC_DECODER);
    }
    /// GB2312 bytes to UTF-16; `ResourceUnicode::convert_gb_to_unicode(b"A", &mut text)`.
    pub fn convert_gb_to_unicode(from: &[u8], to: &mut ExtendedString) {
        *to = multibyte::decode_pairs(from, &multibyte::GB_DECODER);
    }
    /// GBK bytes to UTF-16; false on an invalid byte, Standard_OutOfRange where the native
    /// recovery inserts at position 0; `ResourceUnicode::convert_gbk_to_unicode(b"A", &mut text)?`.
    pub fn convert_gbk_to_unicode(from: &[u8], to: &mut ExtendedString) -> Result<bool, Failure> {
        cjk::decode_into(from, to, cjk::gbk_units)
    }
    /// Big5 bytes to UTF-16, with the same failure modes as GBK;
    /// `ResourceUnicode::convert_big5_to_unicode(b"A", &mut text)?`.
    pub fn convert_big5_to_unicode(from: &[u8], to: &mut ExtendedString) -> Result<bool, Failure> {
        cjk::decode_into(from, to, cjk::big5_units)
    }
    /// UTF-16 to Shift-JIS in a buffer of `maxsize = to.len()` bytes; true when complete;
    /// `ResourceUnicode::convert_unicode_to_sjis(&text, &mut buffer)?`.
    pub fn convert_unicode_to_sjis(from: &ExtendedString, to: &mut [u8]) -> Result<bool, Failure> {
        multibyte::encode_pairs(from, to, &multibyte::SJIS_ENCODER)
    }
    /// UTF-16 to EUC; `ResourceUnicode::convert_unicode_to_euc(&text, &mut buffer)?`.
    pub fn convert_unicode_to_euc(from: &ExtendedString, to: &mut [u8]) -> Result<bool, Failure> {
        multibyte::encode_pairs(from, to, &multibyte::EUC_ENCODER)
    }
    /// UTF-16 to GB2312; `ResourceUnicode::convert_unicode_to_gb(&text, &mut buffer)?`.
    pub fn convert_unicode_to_gb(from: &ExtendedString, to: &mut [u8]) -> Result<bool, Failure> {
        multibyte::encode_pairs(from, to, &multibyte::GB_ENCODER)
    }
    /// UTF-16 to single bytes, spaces above 0xFF; `ResourceUnicode::convert_unicode_to_ansi(&text, &mut buffer)?`.
    pub fn convert_unicode_to_ansi(from: &ExtendedString, to: &mut [u8]) -> Result<bool, Failure> {
        multibyte::encode_ansi(from, to)
    }
    /// Converts bytes in `format` to UTF-16 (native `ConvertFormatToUnicode`); the GBK/Big5
    /// result flag is discarded, as natively;
    /// `ResourceUnicode::convert_format_to_unicode(FormatType::Sjis, bytes, &mut text)?`.
    pub fn convert_format_to_unicode(
        format: FormatType,
        from: &[u8],
        to: &mut ExtendedString,
    ) -> Result<(), Failure> {
        if let Some(row) = format.code_page_index() {
            *to = code_page::decode(row, from);
            return Ok(());
        }
        match format {
            FormatType::Sjis => Self::convert_sjis_to_unicode(from, to),
            FormatType::Euc => Self::convert_euc_to_unicode(from, to),
            FormatType::Gb => Self::convert_gb_to_unicode(from, to),
            FormatType::Gbk => drop(Self::convert_gbk_to_unicode(from, to)?),
            FormatType::Big5 => drop(Self::convert_big5_to_unicode(from, to)?),
            FormatType::Utf8 => *to = decode_multibyte(c_bytes(from)),
            FormatType::SystemLocale => *to = decode_locale(from),
            _ => *to = widen(c_bytes(from)),
        }
        Ok(())
    }
    /// Converts UTF-16 to `format` in a buffer of `maxsize = to.len()` bytes (native
    /// `ConvertUnicodeToFormat`); GBK and Big5 raise Standard_NotImplemented;
    /// `ResourceUnicode::convert_unicode_to_format(FormatType::Utf8, &text, &mut buffer)?`.
    pub fn convert_unicode_to_format(
        format: FormatType,
        from: &ExtendedString,
        to: &mut [u8],
    ) -> Result<bool, Failure> {
        if let Some(row) = format.code_page_index() {
            return code_page::encode(row, from, to);
        }
        match format {
            FormatType::Sjis => Self::convert_unicode_to_sjis(from, to),
            FormatType::Euc => Self::convert_unicode_to_euc(from, to),
            FormatType::Gb => Self::convert_unicode_to_gb(from, to),
            FormatType::Utf8 => encode_utf8(from, to),
            FormatType::SystemLocale => {
                let text = UtfString::<u16>::from_unicode_units(from.as_utf16_with_nul(), -1);
                Ok(text.to_locale(to))
            }
            FormatType::Gbk | FormatType::Big5 => Err(Failure::with_type(
                "Standard_NotImplemented",
                "Resource_Unicode::ConvertUnicodeToFormat - convert from GBK and Big5 to Unocode is not implemented",
                "",
            )),
            _ => Self::convert_unicode_to_ansi(from, to),
        }
    }
}
/// `TCollection_ExtendedString(theFromStr, false)`: every byte widened.
fn widen(bytes: &[u8]) -> ExtendedString {
    ExtendedString::from_raw(
        &bytes
            .iter()
            .map(|&byte| u16::from(byte))
            .collect::<Vec<_>>(),
    )
}
/// `TCollection_ExtendedString(theFromStr, true)`: NCollection_UtfIterator decoding, and the
/// byte-widening fallback at the first symbol above U+10FFFF (surrogates and overlong forms
/// are accepted; symbols with no UTF-16 form are skipped).
///
/// A multi-byte lead directly before the terminator makes the native iterator read past the
/// string; here the missing bytes read as NUL.
fn decode_multibyte(bytes: &[u8]) -> ExtendedString {
    let terminated = [bytes, &[0]].concat();
    let mut iterator = UtfIterator::new(&terminated);
    let mut units = Vec::new();
    while iterator.current() != 0 {
        if !iterator.is_valid() {
            return widen(bytes);
        }
        let mut pair = [0u16; 2];
        let written = iterator
            .get_utf16(&mut pair)
            .expect("two units hold any symbol");
        units.extend_from_slice(&pair[..written]);
        iterator.advance();
    }
    ExtendedString::from_raw(&units)
}
/// `NCollection_UtfString<char16_t>::FromLocale` then `TCollection_ExtendedString(ToCString())`.
fn decode_locale(from: &[u8]) -> ExtendedString {
    let mut text = UtfString::<u16>::new();
    text.from_locale(c_bytes(from), -1);
    ExtendedString::from_utf16(text.to_c_string())
}
/// The UTF-8 branch: false when `maxsize < LengthOfCString()`, else `ToUTF8CString`, which
/// writes the encoded bytes and a terminator (one byte past `maxsize` when they are equal:
/// native overflow, a Standard_OutOfRange here).
fn encode_utf8(from: &ExtendedString, to: &mut [u8]) -> Result<bool, Failure> {
    let units = from.as_utf16_with_nul();
    let mut bytes = Vec::new();
    let mut iterator = UtfIterator::new(units);
    while iterator.current() != 0 {
        let mut buffer = [0u8; 4];
        let written = iterator
            .get_utf8(&mut buffer)
            .expect("four bytes hold any symbol");
        bytes.extend_from_slice(&buffer[..written]);
        iterator.advance();
    }
    if (to.len() as i32) < bytes.len() as i32 {
        return Ok(false);
    }
    for (index, &byte) in bytes.iter().chain(&[0]).enumerate() {
        multibyte::put(to, index as i32, byte)?;
    }
    Ok(true)
}
