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

//! Substring search, bounded location, character sets and tokens.
use super::raise::check_range;
use super::{AsciiString, c_string_length};
use crate::standard::Failure;

impl AsciiString {
    /// `Search`: one-based start of the first occurrence, or -1 when absent or `what` is
    /// empty; `AsciiString::new("abcabc").search("ca") == 3`.
    pub fn search(&self, what: impl AsRef<[u8]>) -> i32 {
        let needle = what.as_ref();
        if needle.is_empty() {
            return -1;
        }
        self.as_bytes()
            .windows(needle.len())
            .position(|part| part == needle)
            .map_or(-1, |index| index as i32 + 1)
    }
    /// `SearchFromEnd`: one-based start of the last occurrence, or -1;
    /// `AsciiString::new("abcabc").search_from_end("ab") == 4`.
    pub fn search_from_end(&self, what: impl AsRef<[u8]>) -> i32 {
        let needle = what.as_ref();
        if needle.is_empty() {
            return -1;
        }
        self.as_bytes()
            .windows(needle.len())
            .rposition(|part| part == needle)
            .map_or(-1, |index| index as i32 + 1)
    }
    /// `Location(const TCollection_AsciiString&, int, int)`: OCCT's scan for `what` inside
    /// `from..=to`, 0 when not found; `AsciiString::new("This is").location("is", 1, 7)? == 3`.
    ///
    /// After a partial match fails the scan restarts at the failing byte, not one byte after
    /// the partial match's start, so an occurrence overlapping a failed prefix is missed:
    /// `"aaab".location("aab", 1, 4)` is 0. An empty string or `what` gives 0 before the
    /// range is checked.
    pub fn location(&self, what: impl AsRef<[u8]>, from: i32, to: i32) -> Result<i32, Failure> {
        let what = what.as_ref();
        if self.is_empty() || what.is_empty() {
            return Ok(0);
        }
        check_range("Location", from, to, self.length())?;
        Ok(scan_location(
            &self.as_bytes()[..to as usize],
            what,
            from as usize - 1,
        ))
    }
    /// `Location(int N, char C, int, int)`: one-based position of the `count`-th `byte`
    /// inside `from..=to`, 0 when there are fewer; `text.location_char(2, b'a', 1, text.length())?`.
    pub fn location_char(&self, count: i32, byte: u8, from: i32, to: i32) -> Result<i32, Failure> {
        check_range("Location", from, to, self.length())?;
        let mut seen = 0;
        for (index, &value) in self
            .as_bytes()
            .iter()
            .enumerate()
            .take(to as usize)
            .skip(from as usize - 1)
        {
            seen += i32::from(value == byte);
            if value == byte && seen == count {
                return Ok(index as i32 + 1);
            }
        }
        Ok(0)
    }
    /// `FirstLocationInSet`: first position in `from..=to` holding a byte of `set`, 0 if none;
    /// an empty string or set gives 0 before the range is checked; `text.first_location_in_set("aeiou", 1, 5)?`.
    pub fn first_location_in_set(
        &self,
        set: impl AsRef<[u8]>,
        from: i32,
        to: i32,
    ) -> Result<i32, Failure> {
        self.first_location_matching("FirstLocationInSet", set.as_ref(), from, to, true)
    }
    /// `FirstLocationNotInSet`: first position in `from..=to` holding no byte of `set`, 0 if
    /// none; `text.first_location_not_in_set(" \t", 1, text.length())?`.
    pub fn first_location_not_in_set(
        &self,
        set: impl AsRef<[u8]>,
        from: i32,
        to: i32,
    ) -> Result<i32, Failure> {
        self.first_location_matching("FirstLocationNotInSet", set.as_ref(), from, to, false)
    }
    fn first_location_matching(
        &self,
        member: &str,
        set: &[u8],
        from: i32,
        to: i32,
        inside: bool,
    ) -> Result<i32, Failure> {
        if self.is_empty() || set.is_empty() {
            return Ok(0);
        }
        check_range(member, from, to, self.length())?;
        let window = &self.as_bytes()[from as usize - 1..to as usize];
        let found = window.iter().position(|byte| set.contains(byte) == inside);
        Ok(found.map_or(0, |offset| from + offset as i32))
    }
    /// `Token`: the `which`-th run of bytes not in the C string `separators` (read through its
    /// first NUL), copied through its first NUL; empty when there are fewer tokens or
    /// `which < 1`; `AsciiString::new("a, b").token(", ", 2).as_bytes() == b"b"`.
    pub fn token(&self, separators: impl AsRef<[u8]>, which: i32) -> Self {
        let separators = separators.as_ref();
        let separators = &separators[..c_string_length(separators)];
        match token_bounds(self.as_bytes(), separators, which) {
            Some((begin, end)) => Self::from_bytes(&self.as_bytes()[begin..end]),
            None => Self::default(),
        }
    }
}

/// OCCT's `Location` loop over `text[start..]`; returns the one-based match start or 0.
fn scan_location(text: &[u8], what: &[u8], start: usize) -> i32 {
    let (mut index, mut matched, mut before) = (start, 0_usize, start as i64 - 1);
    while index < text.len() {
        if text[index] == what[matched] {
            matched += 1;
            if matched == what.len() {
                return before as i32 + 2;
            }
        } else {
            // A failed partial match re-reads the failing byte as a fresh first byte.
            index -= usize::from(matched > 0);
            matched = 0;
            before = index as i64;
        }
        index += 1;
    }
    0
}

/// OCCT's `Token` scan: zero-based `[begin, end)` of the `which`-th token, if any.
fn token_bounds(text: &[u8], separators: &[u8], which: i32) -> Option<(usize, usize)> {
    let mut cursor = 0;
    let mut bounds = None;
    for _ in 0..which.max(0) {
        if cursor == text.len() {
            return None;
        }
        let begin = cursor
            + text[cursor..]
                .iter()
                .position(|byte| !separators.contains(byte))?;
        let end = text[begin..]
            .iter()
            .position(|byte| separators.contains(byte))
            .map_or(text.len(), |offset| begin + offset);
        bounds = Some((begin, end));
        cursor = end;
    }
    bounds
}
