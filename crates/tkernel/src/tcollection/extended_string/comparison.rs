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
impl ExtendedString {
    /// Compares full code-unit content; `text.is_equal(&other)`.
    pub fn is_equal(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() == other.as_ref()
    }
    /// Checks unequal content; `text.is_different(&other)`.
    pub fn is_different(&self, other: impl AsRef<[u16]>) -> bool {
        !self.is_equal(other)
    }
    /// Orders by unsigned UTF-16 units; `text.is_less(&other)`.
    pub fn is_less(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() < other.as_ref()
    }
    /// Orders by unsigned UTF-16 units; `text.is_greater(&other)`.
    pub fn is_greater(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() > other.as_ref()
    }
}
impl ExtendedString {
    /// Compares nullable code units with an explicit signed length; `text.is_equal_span(None,0)`.
    pub fn is_equal_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        if length < 0 || self.len() != length as usize {
            return false;
        }
        if length == 0 {
            return true;
        }
        span(other, length).is_some_and(|units| self.as_utf16() == units)
    }
    /// Negates nullable explicit-length equality; `text.is_different_span(None,5)`.
    pub fn is_different_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        !self.is_equal_span(other, length)
    }
    /// Orders valid spans; invalid spans never compare greater than this string; `text.is_less_span(None,5)`.
    pub fn is_less_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        span(other, length).is_some_and(|units| self.as_utf16() < units)
    }
    /// Orders valid spans while preserving OCCT's invalid-input behavior; `text.is_greater_span(None,-1)`.
    pub fn is_greater_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        if length < 0 {
            return true;
        }
        if other.is_none() {
            return length != 0 || !self.is_empty();
        }
        span(other, length).is_none_or(|units| self.as_utf16() > units)
    }
    /// Tests a bounded nullable prefix; `text.starts_with_span(None,0)`.
    pub fn starts_with_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        if length == 0 {
            return true;
        }
        span(other, length).is_some_and(|units| self.as_utf16().starts_with(units))
    }
    /// Tests a bounded nullable suffix; `text.ends_with_span(None,0)`.
    pub fn ends_with_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        if length == 0 {
            return true;
        }
        span(other, length).is_some_and(|units| self.as_utf16().ends_with(units))
    }
    /// Tests a complete string prefix; `text.starts_with(&prefix)`.
    pub fn starts_with(&self, other: &Self) -> bool {
        self.starts_with_span(Some(other.as_utf16()), other.length())
    }
    /// Tests a complete string suffix; `text.ends_with(&suffix)`.
    pub fn ends_with(&self, other: &Self) -> bool {
        self.ends_with_span(Some(other.as_utf16()), other.length())
    }
}
fn span(other: Option<&[u16]>, length: i32) -> Option<&[u16]> {
    let length = usize::try_from(length).ok()?;
    other?.get(..length)
}
