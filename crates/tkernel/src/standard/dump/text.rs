// Copyright (c) 2019 OPEN CASCADE SAS
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
// Rust translation of the TCollection_AsciiString members Standard_Dump.cxx calls, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One-based `TCollection_AsciiString` primitives the dump parser relies on, with the
//! exceptions they raise, so every parser failure matches the native one.
use super::keys::{JsonKey, json_key_to_string};
use crate::standard::Failure;

/// `Length()` of a byte string; `length(b"ab") == 2`.
pub(super) fn length(text: &[u8]) -> i32 {
    text.len() as i32
}

/// `SubString(from, to)`: the one-based inclusive range, or Standard_OutOfRange;
/// `sub_string(b"abc", 2, 3)? == b"bc"`.
pub(super) fn sub_string(text: &[u8], from: i32, to: i32) -> Result<&[u8], Failure> {
    // The native check compares as unsigned, after rejecting non-positive bounds.
    if from <= 0 || to <= 0 || to as u32 > text.len() as u32 || from as u32 > to as u32 {
        return Err(Failure::out_of_range(
            "TCollection_AsciiString::SubString() out of range",
        ));
    }
    Ok(&text[from as usize - 1..to as usize])
}

/// `Location(what, from, to)`: one-based position of `what` in `from..=to`, 0 when
/// absent, Standard_OutOfRange (no message) for a bad range; `location(b"a, b", b", ", 1, 4)? == 2`.
///
/// The native scan restarts at the mismatching byte instead of after the first matched
/// byte, so a pattern whose prefix repeats (such as "aab" in "aaab") can be missed; the
/// dump keys are one or two distinct bytes, which it always finds.
pub(super) fn location(text: &[u8], what: &[u8], from: i32, to: i32) -> Result<i32, Failure> {
    if text.is_empty() || what.is_empty() {
        return Ok(0);
    }
    if !(to <= length(text) && from > 0 && from <= to) {
        return Err(Failure::out_of_range(""));
    }
    let (mut index, mut matched, mut before) = (from - 1, 1usize, from - 2);
    while index < to && matched <= what.len() {
        if text[index as usize] == what[matched - 1] {
            matched += 1;
        } else {
            // Re-examine the mismatching byte as a possible first byte of the pattern.
            index -= i32::from(matched > 1);
            matched = 1;
            before = index;
        }
        index += 1;
    }
    Ok(if matched > what.len() { before + 2 } else { 0 })
}

/// `Search(what)`: first one-based position, or -1; `search(b"a: b", b": ") == 2`.
pub(super) fn search(text: &[u8], what: &[u8]) -> i32 {
    if what.is_empty() {
        return -1;
    }
    text.windows(what.len())
        .position(|window| window == what)
        .map_or(-1, |index| index as i32 + 1)
}

/// `Location(JsonKeyToString(key), from, Length())`; `find_key(b"a]", JsonKey::CloseContainer, 1)? == 2`.
pub(super) fn find_key(text: &[u8], key: JsonKey, from: i32) -> Result<i32, Failure> {
    location(text, json_key_to_string(key).as_bytes(), from, length(text))
}
