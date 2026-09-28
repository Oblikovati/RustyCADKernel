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

//! Whole-string comparisons; the (pointer, length) forms live in `spans`.
use super::ExtendedString;
impl ExtendedString {
    /// Compares full code-unit content; `text.is_equal(&other)`.
    pub fn is_equal(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() == other.as_ref()
    }
    /// The static two-string IsEqual; `ExtendedString::is_equal_strings(&first, &second)`.
    pub fn is_equal_strings(first: &Self, second: &Self) -> bool {
        first.is_equal(second)
    }
    /// Checks unequal content; `text.is_different(&other)`.
    pub fn is_different(&self, other: impl AsRef<[u16]>) -> bool {
        !self.is_equal(other)
    }
    /// Orders by unsigned code units, a proper prefix first; `text.is_less(&other)`.
    pub fn is_less(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() < other.as_ref()
    }
    /// Orders by unsigned code units, a proper prefix first; `text.is_greater(&other)`.
    pub fn is_greater(&self, other: impl AsRef<[u16]>) -> bool {
        self.as_utf16() > other.as_ref()
    }
    /// Tests a complete prefix; an empty prefix always matches; `text.starts_with(&prefix)`.
    pub fn starts_with(&self, prefix: impl AsRef<[u16]>) -> bool {
        self.as_utf16().starts_with(prefix.as_ref())
    }
    /// Tests a complete suffix; an empty suffix always matches; `text.ends_with(&suffix)`.
    pub fn ends_with(&self, suffix: impl AsRef<[u16]>) -> bool {
        self.as_utf16().ends_with(suffix.as_ref())
    }
    /// Compares, folding only ASCII A-Z when not case sensitive;
    /// `text.is_same_string(&other, false)`.
    pub fn is_same_string(&self, other: &Self, case_sensitive: bool) -> bool {
        if case_sensitive {
            return self.is_equal(other);
        }
        self.len() == other.len()
            && self
                .as_utf16()
                .iter()
                .zip(other.as_utf16())
                .all(|(&first, &second)| lower_ascii(first) == lower_ascii(second))
    }
}
fn lower_ascii(unit: u16) -> u16 {
    if (65..=90).contains(&unit) {
        unit + 32
    } else {
        unit
    }
}
