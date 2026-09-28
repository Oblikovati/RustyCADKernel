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

//! UTF-16 storage used by the byte-string conversion boundary.
/// Owned NUL-terminated UTF-16 code units; `ExtendedString::new("Hello")`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedString {
    terminated: Vec<u16>,
}
impl ExtendedString {
    /// Widens ASCII/byte input, matching OCCT's non-multibyte constructor; `ExtendedString::new("Hello")`.
    pub fn new(text: &str) -> Self {
        Self::from_utf16(
            &text
                .as_bytes()
                .iter()
                .map(|&byte| u16::from(byte))
                .collect::<Vec<_>>(),
        )
    }
    /// Copies UTF-16 through the first zero code unit; `ExtendedString::from_utf16(&[0x20AC,0])`.
    pub fn from_utf16(units: &[u16]) -> Self {
        let mut terminated: Vec<u16> = units
            .iter()
            .copied()
            .take_while(|&unit| unit != 0)
            .collect();
        terminated.push(0);
        Self { terminated }
    }
    /// Returns the stored code units; `text.as_utf16()`.
    pub fn as_utf16(&self) -> &[u16] {
        &self.terminated[..self.len()]
    }
    /// Returns UTF-16 length; `text.len() == 5`.
    pub fn len(&self) -> usize {
        self.terminated.len() - 1
    }
    /// Checks for no code units; `text.is_empty()`.
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
mod comparison;
mod operations;
mod queries;
mod traits;
