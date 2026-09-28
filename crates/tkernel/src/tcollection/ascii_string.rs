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

//! Initial owned, NUL-terminated byte-string operations needed by collection regressions.

/// An owned C-compatible byte string; `AsciiString::new("hello").as_bytes() == b"hello"`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsciiString {
    terminated: Vec<u8>,
}
impl Default for AsciiString {
    fn default() -> Self {
        Self {
            terminated: vec![0],
        }
    }
}
impl AsciiString {
    /// Copies through the first NUL; `AsciiString::new("abc\0def").as_bytes() == b"abc"`.
    pub fn new(text: &str) -> Self {
        Self::from_bytes(text.as_bytes())
    }
    /// Copies arbitrary byte values through the first NUL; `AsciiString::from_bytes(&[255,0])`.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let length = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        let mut terminated = Vec::with_capacity(length + 1);
        terminated.extend_from_slice(&bytes[..length]);
        terminated.push(0);
        Self { terminated }
    }
    /// Formats a signed OCCT integer in decimal; `AsciiString::from_integer(-12).as_bytes() == b"-12"`.
    pub fn from_integer(value: i32) -> Self {
        Self::new(&value.to_string())
    }
    /// Borrows content excluding the terminator; `AsciiString::new("abc").as_bytes() == b"abc"`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.terminated[..self.terminated.len() - 1]
    }
    /// Borrows content including its terminator; `AsciiString::new("abc").as_bytes_with_nul() == b"abc\0"`.
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.terminated
    }
    /// Appends another string's contents; `text.append(&AsciiString::new("suffix"))`.
    pub fn append(&mut self, suffix: &Self) {
        self.terminated.pop();
        self.terminated.extend_from_slice(&suffix.terminated);
    }
}
mod hash;
mod numeric;
mod operations;
mod search;
mod traits;
