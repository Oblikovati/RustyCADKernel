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
impl AsciiString {
    /// Returns byte length, excluding the terminator; `text.len() == 3`.
    pub fn len(&self) -> usize {
        self.terminated.len() - 1
    }
    /// Checks for zero stored length; `AsciiString::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Creates a single nonzero byte; `AsciiString::from_char(b'A')`.
    pub fn from_char(byte: u8) -> Self {
        if byte == 0 {
            Self::default()
        } else {
            Self::from_bytes(&[byte])
        }
    }
    /// Copies at most the requested C-string length; `AsciiString::with_max_length("hello", 3)`.
    pub fn with_max_length(text: &str, maximum: usize) -> Self {
        Self::from_bytes(&text.as_bytes()[..text.len().min(maximum)])
    }
    /// Creates filled storage, including embedded NUL bytes; `AsciiString::filled(5,b'*')`.
    pub fn filled(length: usize, byte: u8) -> Self {
        let mut terminated = vec![byte; length + 1];
        terminated[length] = 0;
        Self { terminated }
    }
    /// Reads a one-based byte position; `text.value(1)`.
    pub fn value(&self, index: usize) -> u8 {
        self.terminated[self.checked_position(index, false)]
    }
    /// Extracts inclusive one-based bounds; `text.substring(3,8)`.
    pub fn substring(&self, first: usize, last: usize) -> Self {
        let start = self.checked_position(first, false);
        let end = self.checked_position(last, false);
        assert!(
            start <= end,
            "AsciiString range {first}..={last}; expected ascending bounds"
        );
        Self::from_raw(&self.as_bytes()[start..=end])
    }
    /// Copies bytes while reusing available capacity; `text.copy("replacement")`.
    pub fn copy(&mut self, input: impl AsRef<[u8]>) {
        let bytes = input.as_ref();
        self.terminated.clear();
        self.terminated.extend_from_slice(bytes);
        self.terminated.push(0);
    }
    /// Appends a converted value; `text.assign_cat(42)`.
    pub fn assign_cat(&mut self, input: impl Into<Self>) {
        self.append(&input.into());
    }
    /// Returns a concatenated copy; `text.cat("suffix")`.
    pub fn cat(&self, input: impl Into<Self>) -> Self {
        let mut result = self.clone();
        result.assign_cat(input);
        result
    }
    /// Inserts before a one-based position; `text.insert(1, "prefix")`.
    pub fn insert(&mut self, index: usize, input: impl Into<Self>) {
        let offset = self.checked_position(index, true);
        let input = input.into();
        self.terminated
            .splice(offset..offset, input.as_bytes().iter().copied());
    }
    /// Inserts a range of this same string without aliasing mutable storage; `text.insert_self(2,0..3)`.
    pub fn insert_self(&mut self, index: usize, source: std::ops::Range<usize>) {
        let copy = Self::from_raw(&self.as_bytes()[source]);
        self.insert(index, copy);
    }
    /// Appends a range from this same string; `text.assign_cat_self(1..text.len())`.
    pub fn assign_cat_self(&mut self, source: std::ops::Range<usize>) {
        let copy = Self::from_raw(&self.as_bytes()[source]);
        self.append(&copy);
    }
    /// Overwrites or extends bytes from a one-based position; `text.set_value(3, "XYZ")`.
    pub fn set_value(&mut self, index: usize, input: impl Into<Self>) {
        let offset = self.checked_position(index, true);
        let input = input.into();
        let end = offset + input.len();
        if end > self.len() {
            self.terminated.resize(end + 1, 0);
        }
        self.terminated[offset..end].copy_from_slice(input.as_bytes());
    }
    /// Removes a byte count from a one-based offset; `text.remove(6,6)`.
    pub fn remove(&mut self, index: usize, count: usize) {
        let offset = self.checked_position(index, true);
        assert!(
            count <= self.len() - offset,
            "AsciiString removal {index}+{count}; expected end <= {}",
            self.len() + 1
        );
        self.terminated.drain(offset..offset + count);
    }
    /// Removes every matching byte; `text.remove_all(b'e')`.
    pub fn remove_all(&mut self, byte: u8) {
        self.terminated.pop();
        self.terminated.retain(|&value| value != byte);
        self.terminated.push(0);
    }
    /// Truncates byte length; `text.truncate(5)`.
    pub fn truncate(&mut self, length: usize) {
        assert!(
            length <= self.len(),
            "AsciiString length {length}; expected 0..={}",
            self.len()
        );
        self.terminated.truncate(length + 1);
        self.terminated[length] = 0;
    }
    /// Keeps a prefix and returns the tail; `let remainder=text.split(5)`.
    pub fn split(&mut self, length: usize) -> Self {
        let remainder = Self::from_raw(&self.as_bytes()[length..]);
        self.truncate(length);
        remainder
    }
    /// Converts ASCII lowercase bytes in place; `text.upper_case()`.
    pub fn upper_case(&mut self) {
        self.terminated.make_ascii_uppercase();
    }
    /// Converts ASCII uppercase bytes in place; `text.lower_case()`.
    pub fn lower_case(&mut self) {
        self.terminated.make_ascii_lowercase();
    }
    /// Uppercases the first byte after lowercasing all bytes; `text.capitalize()`.
    pub fn capitalize(&mut self) {
        self.lower_case();
        self.terminated[0].make_ascii_uppercase();
    }
    /// Moves storage while resetting the source; `AsciiString::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Copies an explicit byte span including embedded NULs; `AsciiString::from_raw(b"a\0b")`.
    pub fn from_raw(bytes: &[u8]) -> Self {
        let mut terminated = bytes.to_vec();
        terminated.push(0);
        Self { terminated }
    }
    fn checked_position(&self, index: usize, allow_end: bool) -> usize {
        let upper = self.len() + usize::from(allow_end);
        assert!(
            index > 0 && index <= upper,
            "AsciiString index {index}; expected 1..={upper}"
        );
        index - 1
    }
}
impl AsciiString {
    /// Copies an overlapping span to the beginning without reallocating; `text.copy_self(1..text.len())`.
    pub fn copy_self(&mut self, source: std::ops::Range<usize>) {
        let length = source.len();
        assert!(
            source.end <= self.len(),
            "AsciiString source {source:?}; expected within 0..{}",
            self.len()
        );
        self.terminated.copy_within(source, 0);
        self.truncate(length);
    }
    /// Compares explicitly bounded byte spans; `AsciiString::same_bytes(b"A",b"a",false)`.
    pub fn same_bytes(first: &[u8], second: &[u8], case_sensitive: bool) -> bool {
        if case_sensitive {
            first == second
        } else {
            first.eq_ignore_ascii_case(second)
        }
    }
}
impl AsciiString {
    /// Returns OCCT's signed byte length; `text.length() == 3`.
    pub fn length(&self) -> i32 {
        self.len() as i32
    }
    /// Replaces storage with a moved source; `text.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
}
impl AsciiString {
    /// Copies a bounded view, stopping at its first NUL like OCCT's view constructor; `AsciiString::from_view(b"a\0b")`.
    pub fn from_view(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}
