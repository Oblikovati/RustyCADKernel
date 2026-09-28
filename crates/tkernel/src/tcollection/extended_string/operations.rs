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

use super::ExtendedString;
use crate::{standard::Failure, tcollection::AsciiString};
impl Default for ExtendedString {
    fn default() -> Self {
        Self {
            terminated: vec![0],
        }
    }
}
impl ExtendedString {
    /// Creates a nonzero UTF-16 code unit; `ExtendedString::from_char(0x20AC)`.
    pub fn from_char(unit: u16) -> Self {
        Self::from_utf16(&[unit])
    }
    /// Creates a filled string or rejects negative lengths; `ExtendedString::filled(5,88)?`.
    pub fn filled(length: i32, unit: u16) -> Result<Self, Failure> {
        if length < 0 {
            return Err(Failure::out_of_range(&format!(
                "ExtendedString length {length}; expected >= 0"
            )));
        }
        let mut terminated = vec![unit; length as usize + 1];
        terminated[length as usize] = 0;
        Ok(Self { terminated })
    }
    /// Decodes valid UTF-8 into code units; `ExtendedString::from_utf8("é")`.
    pub fn from_utf8(text: &str) -> Self {
        Self::from_utf16(&text.encode_utf16().collect::<Vec<_>>())
    }
    /// Converts an ASCII byte string using OCCT's default UTF-8 detection; `ExtendedString::from_ascii(&ascii)`.
    pub fn from_ascii(ascii: &AsciiString) -> Self {
        let bytes = ascii.as_bytes();
        let end = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());
        match std::str::from_utf8(&bytes[..end]) {
            Ok(text) => Self::from_utf8(text),
            Err(_) => Self::from_raw(
                &bytes
                    .iter()
                    .map(|&byte| u16::from(byte))
                    .collect::<Vec<_>>(),
            ),
        }
    }
    /// Converts wide scalar input through its first zero; `ExtendedString::from_wide(&[0x20AC,0])`.
    pub fn from_wide(units: &[u32]) -> Self {
        let text: String = units
            .iter()
            .copied()
            .take_while(|&unit| unit != 0)
            .filter_map(char::from_u32)
            .collect();
        Self::from_utf8(&text)
    }
    /// Formats an OCCT integer; `ExtendedString::from_integer(42)`.
    pub fn from_integer(value: i32) -> Self {
        Self::from_ascii(&AsciiString::from_integer(value))
    }
    /// Formats six significant decimal digits; `ExtendedString::from_real(3.14)`.
    pub fn from_real(value: f64) -> Self {
        Self::from_ascii(&AsciiString::from_real(value))
    }
    /// Copies bounded code units including zero units; `ExtendedString::from_raw(&[65,0,66])`.
    pub fn from_raw(units: &[u16]) -> Self {
        let mut terminated = units.to_vec();
        terminated.push(0);
        Self { terminated }
    }
    /// Copies bounded code units while reusing capacity; `text.copy(&[65,66])`.
    pub fn copy(&mut self, units: &[u16]) {
        self.terminated.clear();
        self.terminated.extend_from_slice(units);
        self.terminated.push(0);
    }
    /// Returns the signed UTF-16 length; `text.length()==5`.
    pub fn length(&self) -> i32 {
        self.len() as i32
    }
    /// Borrows terminated code units; `text.as_utf16_with_nul().last()==Some(&0)`.
    pub fn as_utf16_with_nul(&self) -> &[u16] {
        &self.terminated
    }
    /// Reads a one-based code unit; `text.value(1)`.
    pub fn value(&self, index: usize) -> u16 {
        assert!(
            index > 0 && index <= self.len(),
            "ExtendedString index {index}; expected 1..={}",
            self.len()
        );
        self.terminated[index - 1]
    }
    /// Tests OCCT's eight-bit ASCII range; `ExtendedString::from_char(255).is_ascii()`.
    pub fn is_ascii(&self) -> bool {
        self.as_utf16().iter().all(|&unit| unit <= 255)
    }
    /// Appends converted input; `text.assign_cat(42)`.
    pub fn assign_cat(&mut self, input: impl Into<Self>) {
        let input = input.into();
        self.terminated.pop();
        self.terminated.extend_from_slice(&input.terminated);
    }
    /// Returns an appended copy; `text.cat(b'!')`.
    pub fn cat(&self, input: impl Into<Self>) -> Self {
        let mut copy = self.clone();
        copy.assign_cat(input);
        copy
    }
    /// Inserts converted input at a one-based boundary; `text.insert(1,0x20AC_u16)`.
    pub fn insert(&mut self, index: usize, input: impl Into<Self>) {
        assert!(
            index > 0 && index <= self.len() + 1,
            "ExtendedString index {index}; expected 1..={}",
            self.len() + 1
        );
        let input = input.into();
        self.terminated
            .splice(index - 1..index - 1, input.as_utf16().iter().copied());
    }
    /// Prepends converted input; `text.prepend(ExtendedString::new("Hello "))`.
    pub fn prepend(&mut self, input: impl Into<Self>) {
        self.insert(1, input);
    }
    /// Appends a self-overlapping span safely; `text.assign_cat_self(0..text.len())`.
    pub fn assign_cat_self(&mut self, range: std::ops::Range<usize>) {
        let copy = Self::from_raw(&self.as_utf16()[range]);
        self.assign_cat(copy);
    }
    /// Prepends a self-overlapping span safely; `text.prepend_self(5..10)`.
    pub fn prepend_self(&mut self, range: std::ops::Range<usize>) {
        let copy = Self::from_raw(&self.as_utf16()[range]);
        self.prepend(copy);
    }
    /// Removes a count from a one-based position; `text.remove(6,6)`.
    pub fn remove(&mut self, index: usize, count: usize) {
        assert!(
            index > 0 && index - 1 + count <= self.len(),
            "ExtendedString removal {index}+{count}; expected within 1..={}",
            self.len()
        );
        self.terminated.drain(index - 1..index - 1 + count);
    }
    /// Replaces every matching code unit; `text.change_all(111,88)`.
    pub fn change_all(&mut self, from: u16, to: u16) {
        let length = self.len();
        for unit in &mut self.terminated[..length] {
            if *unit == from {
                *unit = to;
            }
        }
    }
    /// Returns encoded UTF-8 byte length through the first zero; `text.utf8_length()`.
    pub fn utf8_length(&self) -> usize {
        self.to_utf8().len()
    }
    /// Writes UTF-8 and its NUL into a caller buffer; `text.write_utf8(&mut buffer)?`.
    pub fn write_utf8(&self, buffer: &mut [u8]) -> Result<usize, Failure> {
        let encoded = self.to_utf8();
        let length = encoded.len();
        if buffer.len() <= length {
            return Err(Failure::out_of_range(&format!(
                "UTF-8 buffer {}; expected at least {} bytes",
                buffer.len(),
                length + 1
            )));
        }
        buffer[..length].copy_from_slice(encoded.as_bytes());
        buffer[length] = 0;
        Ok(length)
    }
    /// Returns UTF-8 through the first zero code unit; `text.to_utf8()`.
    pub fn to_utf8(&self) -> String {
        let units = self.as_utf16();
        let end = units
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(units.len());
        char::decode_utf16(units[..end].iter().copied())
            .filter_map(Result::ok)
            .collect()
    }
    /// Hashes native-endian UTF-16 bytes with OCCT's algorithm; `text.hash_code()`.
    pub fn hash_code(&self) -> usize {
        AsciiString::from_raw(
            &self
                .as_utf16()
                .iter()
                .flat_map(|unit| unit.to_ne_bytes())
                .collect::<Vec<_>>(),
        )
        .hash_code()
    }
}
