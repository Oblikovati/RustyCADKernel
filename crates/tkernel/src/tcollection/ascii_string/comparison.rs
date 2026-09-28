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

//! Equality, ordering, prefix/suffix tests and the ASCII check, all over full byte spans.
//!
//! OCCT compares with `memcmp`, so bytes order as unsigned values and a shorter common
//! prefix orders first; this is the order of Rust byte slices.
use super::AsciiString;

impl AsciiString {
    /// `IsEqual`/`operator==`: same length and bytes; `AsciiString::new("Test").is_equal("Test")`.
    pub fn is_equal(&self, other: impl AsRef<[u8]>) -> bool {
        self.as_bytes() == other.as_ref()
    }
    /// `IsDifferent`/`operator!=`; `AsciiString::new("Test").is_different("Other")`.
    pub fn is_different(&self, other: impl AsRef<[u8]>) -> bool {
        !self.is_equal(other)
    }
    /// `IsLess`/`operator<`: `memcmp` over the common length, then the shorter is less;
    /// `AsciiString::new("A").is_less("B")`.
    pub fn is_less(&self, other: impl AsRef<[u8]>) -> bool {
        self.as_bytes() < other.as_ref()
    }
    /// `IsGreater`/`operator>`; `AsciiString::new("B").is_greater("A")`.
    pub fn is_greater(&self, other: impl AsRef<[u8]>) -> bool {
        self.as_bytes() > other.as_ref()
    }
    /// `StartsWith`: true for an empty prefix; `AsciiString::new("Hello").starts_with("He")`.
    pub fn starts_with(&self, prefix: impl AsRef<[u8]>) -> bool {
        self.as_bytes().starts_with(prefix.as_ref())
    }
    /// `EndsWith`: true for an empty suffix; `AsciiString::new("Hello").ends_with("lo")`.
    pub fn ends_with(&self, suffix: impl AsRef<[u8]>) -> bool {
        self.as_bytes().ends_with(suffix.as_ref())
    }
    /// Member form of `IsSameString(*this, other, case)`; `text.is_same_string("TEST", false)`.
    pub fn is_same_string(&self, other: impl AsRef<[u8]>, case_sensitive: bool) -> bool {
        Self::same_bytes(self, other, case_sensitive)
    }
    /// Static `IsSameString`: equal lengths and bytes, optionally ignoring ASCII case;
    /// `AsciiString::same_bytes("A", "a", false)`.
    pub fn same_bytes(
        first: impl AsRef<[u8]>,
        second: impl AsRef<[u8]>,
        case_sensitive: bool,
    ) -> bool {
        let (first, second) = (first.as_ref(), second.as_ref());
        if case_sensitive {
            first == second
        } else {
            first.eq_ignore_ascii_case(second)
        }
    }
    /// Static `IsEqual(a, b)`: byte-span equality of two strings or views;
    /// `AsciiString::strings_equal(&text, "Test")`.
    pub fn strings_equal(first: impl AsRef<[u8]>, second: impl AsRef<[u8]>) -> bool {
        first.as_ref() == second.as_ref()
    }
    /// `IsAscii`: every byte is printable ASCII (32..=126); `AsciiString::new("a b").is_ascii()`.
    pub fn is_ascii(&self) -> bool {
        self.as_bytes()
            .iter()
            .all(|&byte| (b' '..127).contains(&byte))
    }
}
