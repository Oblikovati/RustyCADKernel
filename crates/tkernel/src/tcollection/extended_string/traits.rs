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
impl AsRef<[u16]> for ExtendedString {
    fn as_ref(&self) -> &[u16] {
        self.as_utf16()
    }
}
impl From<&ExtendedString> for ExtendedString {
    fn from(text: &ExtendedString) -> Self {
        text.clone()
    }
}
impl From<&[u16]> for ExtendedString {
    fn from(units: &[u16]) -> Self {
        Self::from_raw(units)
    }
}
impl From<u8> for ExtendedString {
    fn from(byte: u8) -> Self {
        Self::from_char(u16::from(byte))
    }
}
impl From<u16> for ExtendedString {
    fn from(unit: u16) -> Self {
        Self::from_char(unit)
    }
}
impl From<i32> for ExtendedString {
    fn from(value: i32) -> Self {
        Self::from_integer(value)
    }
}
impl From<f64> for ExtendedString {
    fn from(value: f64) -> Self {
        Self::from_real(value)
    }
}
impl<T: Into<ExtendedString>> std::ops::Add<T> for &ExtendedString {
    type Output = ExtendedString;
    fn add(self, other: T) -> Self::Output {
        self.cat(other)
    }
}
impl<T: Into<ExtendedString>> std::ops::AddAssign<T> for ExtendedString {
    fn add_assign(&mut self, other: T) {
        self.assign_cat(other);
    }
}
impl PartialOrd for ExtendedString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ExtendedString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_utf16().cmp(other.as_utf16())
    }
}
impl crate::ncollection::HashKey for ExtendedString {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}
