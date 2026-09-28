// Copyright (c) 1993-1999 Matra Datavision
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

//! UTF-8 output, printing, raw access and HashCode.
use super::{ExtendedString, errors::out_of_range, utf};
use crate::standard::{Failure, hash_bytes};
use std::io;
impl ExtendedString {
    /// Returns UTF-8 through the first zero code unit, dropping unpaired surrogates;
    /// `text.to_utf8()`.
    pub fn to_utf8(&self) -> String {
        utf::utf16_to_utf8(self.as_utf16())
    }
    /// Returns the UTF-8 byte length `to_utf8` produces (`LengthOfCString`);
    /// `text.utf8_length()`.
    pub fn utf8_length(&self) -> usize {
        self.to_utf8().len()
    }
    /// Writes UTF-8 and a NUL into a caller buffer and returns the byte count
    /// (`ToUTF8CString`). OCCT trusts the buffer; the port refuses one shorter than
    /// `utf8_length() + 1` with Standard_OutOfRange; `text.write_utf8(&mut buffer)?`.
    pub fn write_utf8(&self, buffer: &mut [u8]) -> Result<usize, Failure> {
        let encoded = self.to_utf8();
        let length = encoded.len();
        if buffer.len() <= length {
            return Err(out_of_range(
                "TCollection_ExtendedString::ToUTF8CString : buffer too small",
                format_args!(
                    "buffer {} bytes; expected at least {}",
                    buffer.len(),
                    length + 1
                ),
            ));
        }
        buffer[..length].copy_from_slice(encoded.as_bytes());
        buffer[length] = 0;
        Ok(length)
    }
    /// Borrows the units with their terminating zero (`ToExtString`);
    /// `text.as_utf16_with_nul().last() == Some(&0)`.
    pub fn as_utf16_with_nul(&self) -> &[u16] {
        &self.terminated
    }
    /// Windows-only view of the units as a UTF-16 `wchar_t` string (`ToWideString`);
    /// `text.to_wide_string()`.
    #[cfg(windows)]
    pub fn to_wide_string(&self) -> &[u16] {
        self.as_utf16_with_nul()
    }
    /// Writes the UTF-8 form through the first zero unit (`Print`, `operator<<`);
    /// `text.print(&mut std::io::stdout())?`.
    pub fn print(&self, output: &mut impl io::Write) -> io::Result<()> {
        output.write_all(self.to_utf8().as_bytes())
    }
    /// OCCT's `HashCode`: FNV-1a over the native-endian unit bytes (signed-char quirk) for
    /// fewer than two units, Murmur otherwise; equal to the AsciiString hash of the same
    /// ASCII text; `text.hash_code()`.
    pub fn hash_code(&self) -> usize {
        let bytes: Vec<u8> = self
            .as_utf16()
            .iter()
            .flat_map(|unit| unit.to_ne_bytes())
            .collect();
        if self.len() < 2 {
            return fnv_signed_bytes(&bytes);
        }
        hash_bytes(&bytes)
    }
}
impl std::fmt::Display for ExtendedString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_utf8())
    }
}
/// `opencascade::FNVHash::hash_combine` at the native width; OCCT reads the bytes through
/// a (signed) `char*`, so bytes >= 0x80 are sign-extended before the xor.
fn fnv_signed_bytes(bytes: &[u8]) -> usize {
    if cfg!(target_pointer_width = "64") {
        let fold = |hash: u64, &byte: &u8| {
            (hash ^ (byte as i8 as i64 as u64)).wrapping_mul(1_099_511_628_211)
        };
        return bytes.iter().fold(14_695_981_039_346_656_037, fold) as usize;
    }
    let fold = |hash: u32, &byte: &u8| (hash ^ (byte as i8 as i32 as u32)).wrapping_mul(16_777_619);
    bytes.iter().fold(2_166_136_261, fold) as usize
}
