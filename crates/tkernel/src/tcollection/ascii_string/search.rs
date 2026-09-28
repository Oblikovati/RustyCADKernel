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
    /// Finds the first occurrence, returning -1 when absent; `text.search("test")`.
    pub fn search(&self, input: impl AsRef<[u8]>) -> i32 {
        let needle = input.as_ref();
        if needle.is_empty() {
            return -1;
        }
        self.as_bytes()
            .windows(needle.len())
            .position(|part| part == needle)
            .map_or(-1, |index| index as i32 + 1)
    }
    /// Finds the final occurrence, returning -1 when absent; `text.search_from_end("test")`.
    pub fn search_from_end(&self, input: impl AsRef<[u8]>) -> i32 {
        let needle = input.as_ref();
        if needle.is_empty() {
            return -1;
        }
        self.as_bytes()
            .windows(needle.len())
            .rposition(|part| part == needle)
            .map_or(-1, |index| index as i32 + 1)
    }
    /// Searches an inclusive byte interval; `text.location("i", 4, text.len())`.
    pub fn location(&self, input: impl AsRef<[u8]>, first: usize, last: usize) -> i32 {
        let result = self.substring(first, last).search(input);
        if result < 0 {
            0
        } else {
            result + first as i32 - 1
        }
    }
    /// Returns a one-based token separated by any delimiter byte; `text.token(" ,", 2)`.
    pub fn token(&self, separators: &str, index: usize) -> Self {
        let token = self
            .as_bytes()
            .split(|byte| separators.as_bytes().contains(byte))
            .filter(|part| !part.is_empty())
            .nth(index.saturating_sub(1))
            .unwrap_or(&[]);
        Self::from_raw(token)
    }
    /// Compares complete stored bytes; `text.is_equal("Test")`.
    pub fn is_equal(&self, other: impl AsRef<[u8]>) -> bool {
        self.as_bytes() == other.as_ref()
    }
    /// Checks unequal stored bytes; `text.is_different("Other")`.
    pub fn is_different(&self, other: impl AsRef<[u8]>) -> bool {
        !self.is_equal(other)
    }
    /// Lexicographically compares byte values; `text.is_less("Z")`.
    pub fn is_less(&self, other: impl AsRef<[u8]>) -> bool {
        self.as_bytes() < other.as_ref()
    }
    /// Lexicographically compares byte values; `text.is_greater("A")`.
    pub fn is_greater(&self, other: impl AsRef<[u8]>) -> bool {
        self.as_bytes() > other.as_ref()
    }
    /// Optionally ignores ASCII case; `text.is_same_string("TEST", false)`.
    pub fn is_same_string(&self, other: impl AsRef<[u8]>, case_sensitive: bool) -> bool {
        if case_sensitive {
            self.is_equal(other)
        } else {
            self.as_bytes().eq_ignore_ascii_case(other.as_ref())
        }
    }
    /// Returns the last graphic byte boundary, preserving non-ASCII UTF-8; `text.useful_length()`.
    pub fn useful_length(&self) -> usize {
        let bytes = self.as_bytes();
        let end = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());
        bytes[..end]
            .iter()
            .rposition(|byte| byte.is_ascii_graphic() || *byte >= 128)
            .map_or(0, |index| index + 1)
    }
}
