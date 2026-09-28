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
impl ExtendedString {
    /// Finds a member of a code-unit set within inclusive bounds; `text.first_location_in_set(&set,1,text.len())`.
    pub fn first_location_in_set(&self, set: &Self, first: usize, last: usize) -> usize {
        self.locate_set(set, first, last, true)
    }
    /// Finds a code unit outside a set within inclusive bounds; `text.first_location_not_in_set(&set,1,text.len())`.
    pub fn first_location_not_in_set(&self, set: &Self, first: usize, last: usize) -> usize {
        self.locate_set(set, first, last, false)
    }
    fn locate_set(&self, set: &Self, first: usize, last: usize, inside: bool) -> usize {
        assert!(
            first > 0 && first <= last && last <= self.len(),
            "ExtendedString interval {first}..={last}; expected within 1..={}",
            self.len()
        );
        self.as_utf16()[first - 1..last]
            .iter()
            .position(|unit| set.as_utf16().contains(unit) == inside)
            .map_or(0, |index| index + first)
    }
    /// Parses a decimal prefix, returning zero for empty strings; `text.integer_value()?`.
    pub fn integer_value(&self) -> Result<i32, Failure> {
        if self.is_empty() {
            Ok(0)
        } else {
            AsciiString::from_extended(self, None).integer_value()
        }
    }
    /// Checks for a decimal integer prefix; `text.is_integer_value()`.
    pub fn is_integer_value(&self) -> bool {
        !self.is_empty() && AsciiString::from_extended(self, None).is_integer_value()
    }
    /// Parses a real prefix, returning zero for empty strings; `text.real_value()?`.
    pub fn real_value(&self) -> Result<f64, Failure> {
        if self.is_empty() {
            Ok(0.0)
        } else {
            AsciiString::from_extended(self, None).real_value()
        }
    }
    /// Checks a real prefix with optional full consumption; `text.is_real_value(true)`.
    pub fn is_real_value(&self, full: bool) -> bool {
        !self.is_empty() && AsciiString::from_extended(self, None).is_real_value(full)
    }
    /// Compares with optional ASCII case folding; `text.is_same_string(&other,false)`.
    pub fn is_same_string(&self, other: &Self, case_sensitive: bool) -> bool {
        if case_sensitive {
            return self.is_equal(other);
        }
        self.len() == other.len()
            && self
                .as_utf16()
                .iter()
                .zip(other.as_utf16())
                .all(|(&a, &b)| lower_ascii(a) == lower_ascii(b))
    }
}
fn lower_ascii(unit: u16) -> u16 {
    if (65..=90).contains(&unit) {
        unit + 32
    } else {
        unit
    }
}
