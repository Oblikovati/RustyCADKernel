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

//! The (pointer, length) comparison and search cores with OCCT's null/negative answers.
//!
//! Every core checks its arguments in OCCT's order and reads only the units OCCT reads, so
//! a span shorter than the length is accepted wherever OCCT would not touch the missing
//! units; reading one of them panics (native undefined behavior).
use super::{ExtendedString, errors::leading_units};
use std::cmp::Ordering;
impl ExtendedString {
    /// IsEqual(ptr, length): a negative length or a null non-empty span is unequal;
    /// `text.is_equal_span(None, 0)`.
    pub fn is_equal_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        if length < 0 || self.length() != length {
            return false;
        }
        if length == 0 {
            return true;
        }
        other.is_some_and(|units| self.as_utf16() == leading_units(units, self.len()))
    }
    /// IsDifferent(ptr, length), the exact negation of the span IsEqual;
    /// `text.is_different_span(None, 5)`.
    pub fn is_different_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        !self.is_equal_span(other, length)
    }
    /// IsLess(ptr, length): false for a negative length or a null pointer;
    /// `text.is_less_span(Some(&[66]), 1)`.
    pub fn is_less_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        let (Some(units), Ok(count)) = (other, usize::try_from(length)) else {
            return false;
        };
        self.compare_span(units, count) == Ordering::Less
    }
    /// IsGreater(ptr, length): true for a negative length; a null pointer counts as empty
    /// only when the length is zero; `text.is_greater_span(None, 0)`.
    pub fn is_greater_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        let Ok(count) = usize::try_from(length) else {
            return true;
        };
        match other {
            None => count > 0 || !self.is_empty(),
            Some(units) => self.compare_span(units, count) == Ordering::Greater,
        }
    }
    /// StartsWith(ptr, length): a zero length matches, a negative length or null pointer
    /// does not; `text.starts_with_span(Some(&[72]), 1)`.
    pub fn starts_with_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        self.affix_span(other, length)
            .is_some_and(|units| self.as_utf16().starts_with(units))
    }
    /// EndsWith(ptr, length), with StartsWith's argument rules; `text.ends_with_span(None, 0)`.
    pub fn ends_with_span(&self, other: Option<&[u16]>, length: i32) -> bool {
        self.affix_span(other, length)
            .is_some_and(|units| self.as_utf16().ends_with(units))
    }
    /// Search(ptr, length): -1 for a null pointer, a length <= 0 or longer than the string;
    /// `text.search_span(Some(&[108]), 1)`.
    pub fn search_span(&self, what: Option<&[u16]>, length: i32) -> i32 {
        self.search_argument(what, length)
            .map_or(-1, |units| self.search(units))
    }
    /// SearchFromEnd(ptr, length), with Search's argument rules;
    /// `text.search_from_end_span(Some(&[108]), 1)`.
    pub fn search_from_end_span(&self, what: Option<&[u16]>, length: i32) -> i32 {
        self.search_argument(what, length)
            .map_or(-1, |units| self.search_from_end(units))
    }
    /// OCCT's IsLess/IsGreater loop: unsigned units up to the shorter length, then lengths.
    fn compare_span(&self, units: &[u16], count: usize) -> Ordering {
        let shared = self.len().min(count);
        for (index, &unit) in self.as_utf16()[..shared].iter().enumerate() {
            let other = leading_units(units, index + 1)[index];
            if unit != other {
                return unit.cmp(&other);
            }
        }
        self.len().cmp(&count)
    }
    /// The units StartsWith/EndsWith compare: `Some(&[])` for a zero length, `None` when the
    /// answer is false before any unit is read.
    fn affix_span<'a>(&self, other: Option<&'a [u16]>, length: i32) -> Option<&'a [u16]> {
        let count = usize::try_from(length).ok()?;
        if count == 0 {
            return Some(&[]);
        }
        self.bounded_span(other?, count)
    }
    /// The needle Search/SearchFromEnd look for, or `None` when OCCT returns -1 unread.
    fn search_argument<'a>(&self, what: Option<&'a [u16]>, length: i32) -> Option<&'a [u16]> {
        let count = usize::try_from(length).ok().filter(|&count| count > 0)?;
        self.bounded_span(what?, count)
    }
    /// The first `count` units, or `None` when they cannot fit in this string (OCCT answers
    /// before reading them).
    fn bounded_span<'a>(&self, units: &'a [u16], count: usize) -> Option<&'a [u16]> {
        (count <= self.len()).then(|| leading_units(units, count))
    }
}
