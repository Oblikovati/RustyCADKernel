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

use super::AsciiString;
impl AsRef<[u8]> for AsciiString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
impl From<&str> for AsciiString {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}
impl From<&AsciiString> for AsciiString {
    fn from(text: &AsciiString) -> Self {
        text.clone()
    }
}
impl From<u8> for AsciiString {
    fn from(byte: u8) -> Self {
        Self::from_char(byte)
    }
}
impl From<i32> for AsciiString {
    fn from(value: i32) -> Self {
        Self::from_integer(value)
    }
}
impl From<f64> for AsciiString {
    fn from(value: f64) -> Self {
        Self::from_real(value)
    }
}
impl<T: Into<AsciiString>> std::ops::Add<T> for &AsciiString {
    type Output = AsciiString;
    fn add(self, other: T) -> AsciiString {
        self.cat(other)
    }
}
impl<T: Into<AsciiString>> std::ops::AddAssign<T> for AsciiString {
    fn add_assign(&mut self, other: T) {
        self.assign_cat(other);
    }
}
impl PartialEq<&str> for AsciiString {
    fn eq(&self, other: &&str) -> bool {
        self.is_equal(other)
    }
}
impl PartialOrd for AsciiString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for AsciiString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}
impl From<&[u8]> for AsciiString {
    fn from(bytes: &[u8]) -> Self {
        Self::from_raw(bytes)
    }
}
