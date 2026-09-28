// Copyright (c) 2025 OPEN CASCADE SAS
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

//! `TCollection_ExtendedString`: an owned sequence of UTF-16 code units.
//!
//! Lengths and indices count 16-bit code units, as in OCCT: a surrogate pair counts twice.
//! Members that raise an OCCT exception return `Result<_, Failure>` with the same class.
//! A C++ `const char16_t*` argument becomes a slice that ends where the C string ends; a
//! (pointer, length) pair keeps the nullable pointer as `Option<&[u16]>`.
/// Owned NUL-terminated UTF-16 code units; `ExtendedString::new("Hello")`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedString {
    terminated: Vec<u16>,
}
impl ExtendedString {
    /// Widens each byte up to the first NUL (OCCT's non-multibyte `const char*` constructor);
    /// `ExtendedString::new("Hello")`.
    pub fn new(text: &str) -> Self {
        Self::from_c_string(text.as_bytes(), false)
    }
    /// Copies UTF-16 through the first zero code unit (the `const char16_t*` constructor);
    /// `ExtendedString::from_utf16(&[0x20AC, 0])`.
    pub fn from_utf16(units: &[u16]) -> Self {
        let end = units
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(units.len());
        Self::from_raw(&units[..end])
    }
    /// Returns the stored code units without the terminator; `text.as_utf16()`.
    pub fn as_utf16(&self) -> &[u16] {
        &self.terminated[..self.len()]
    }
    /// Returns the length in code units; `ExtendedString::new("Hello").len() == 5`.
    pub fn len(&self) -> usize {
        self.terminated.len() - 1
    }
    /// Checks for no code units (`IsEmpty`); `ExtendedString::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl super::AsciiString {
    /// Converts UTF-16 to UTF-8, or replaces non-ASCII units; `AsciiString::from_extended(&text,Some(b'?'))`.
    pub fn from_extended(text: &ExtendedString, replacement: Option<u8>) -> Self {
        if let Some(byte) = replacement.filter(|&byte| byte != 0) {
            return Self::from_raw(
                &text
                    .as_utf16()
                    .iter()
                    .map(|&unit| if unit <= 255 { unit as u8 } else { byte })
                    .collect::<Vec<_>>(),
            );
        }
        Self::new(&text.to_utf8())
    }
}
impl From<&ExtendedString> for super::AsciiString {
    fn from(text: &ExtendedString) -> Self {
        Self::from_extended(text, None)
    }
}
impl super::AsciiString {
    /// Appends wide Unicode scalar input through its first zero; `text.assign_cat_wide(&[32,0x20AC,0])`.
    pub fn assign_cat_wide(&mut self, units: &[u32]) {
        let text: String = units
            .iter()
            .copied()
            .take_while(|&unit| unit != 0)
            .filter_map(char::from_u32)
            .collect();
        self.assign_cat(text.as_str());
    }
    /// Returns a copy with wide scalar input appended; `text.cat_wide(&[32,87,0])`.
    pub fn cat_wide(&self, units: &[u32]) -> Self {
        let mut result = self.clone();
        result.assign_cat_wide(units);
        result
    }
}
mod adjustment;
mod append;
mod comparison;
mod construction;
mod conversion;
mod editing;
mod errors;
mod queries;
mod search;
mod spans;
mod traits;
mod utf;
