// Created on: 1992-06-24
// Created by: Gilles DEBARBOUILLE
// Copyright (c) 1992-1999 Matra Datavision
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

//! Shared helpers reproducing the C string and NCollection_Sequence behavior the
//! Units classes rely on, including the native exception messages.
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle, RuntimeTyped};
use std::cmp::Ordering;

/// Bytes of a C string argument: everything before the first NUL.
pub(crate) fn c_str(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..end]
}

/// `strncmp(left, right, count)` over NUL-terminated byte strings, as unsigned bytes.
pub(crate) fn strncmp(left: &[u8], right: &[u8], count: usize) -> Ordering {
    for index in 0..count {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        if a != b || a == 0 {
            return a.cmp(&b);
        }
    }
    Ordering::Equal
}

/// Borrows a handle's object, reporting the native null dereference as an error.
///
/// Native OCCT dereferences such handles unchecked (undefined behaviour); Rust reports
/// `Standard_NullObject` naming the member that would have crashed.
pub(crate) fn object<'a, T: RuntimeTyped>(
    handle: &'a Handle<T>,
    member: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            &format!("{member}: null handle; expected a constructed object"),
            "",
        )
    })
}

/// `NCollection_Sequence::Value(index)` with the native one-based range check message.
pub(crate) fn item<T: Clone + Send + Sync + 'static>(
    sequence: &Handle<HSequence<T>>,
    index: usize,
) -> Result<T, Failure> {
    let items = object(sequence, "NCollection_HSequence")?;
    items
        .sequence(|values| values.value(index).cloned())
        .map_err(|_| Failure::out_of_range("NCollection_Sequence::Value"))
}

/// Number of items of a handled sequence; `length(&sequence)?`.
pub(crate) fn length<T: Send + Sync + 'static>(
    sequence: &Handle<HSequence<T>>,
) -> Result<usize, Failure> {
    Ok(object(sequence, "NCollection_HSequence")?.sequence(|values| values.len()))
}

/// Runs an edit on a handled sequence, mapping range failures to `message`.
pub(crate) fn edit<T: Send + Sync + 'static, R>(
    sequence: &Handle<HSequence<T>>,
    message: &str,
    change: impl FnOnce(&mut crate::ncollection::Sequence<T>) -> Result<R, Failure>,
) -> Result<R, Failure> {
    let items = object(sequence, "NCollection_HSequence")?;
    items
        .change_sequence(change)
        .map_err(|_| Failure::out_of_range(message))
}

/// A new, empty handled sequence; `new_sequence::<i32>()`.
pub(crate) fn new_sequence<T: Send + Sync + 'static>() -> Handle<HSequence<T>> {
    Handle::new(HSequence::default())
}

/// `NCollection_BaseSequence::removeSeq` single-index message.
pub(crate) const REMOVE_ONE: &str = "NCollection_BaseSequence::removeSeq() - index is out of range";
/// `NCollection_BaseSequence::removeSeq` range message.
pub(crate) const REMOVE_RANGE: &str = "NCollection_BaseSequence::removeSeq() - invalid input range";
/// `NCollection_Sequence::ChangeValue` (used by `SetValue`) message.
pub(crate) const CHANGE_VALUE: &str = "NCollection_Sequence::ChangeValue";
/// `NCollection_BaseSequence::PExchange` raises with an empty message.
pub(crate) const EXCHANGE: &str = "";
/// `InsertBefore`/`InsertAfter` messages for out-of-range positions.
pub(crate) const INSERT_AFTER: &str = "NCollection_Sequence::InsertAfter";

/// `std::cout << double`: the default stream formatting, C `%g` with six digits.
pub(crate) fn stream_real(value: f64) -> String {
    String::from_utf8_lossy(crate::tcollection::AsciiString::from_real(value).as_bytes()).into()
}

/// `TCollection_AsciiString::RightAdjust` / `strrightadjust`: drops trailing C spaces.
pub(crate) fn right_adjust(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .rposition(|byte| !crate::standard::character::is_space(*byte))
        .map_or(0, |last| last + 1);
    &bytes[..end]
}

/// `OSD::CStringToReal`: the value `Strtod` assigns, whether or not text remains.
pub(crate) fn c_string_to_real(bytes: &[u8]) -> f64 {
    let bytes = c_str(bytes);
    // A non-ASCII byte cannot extend a C-locale number, so parse the ASCII prefix only.
    let end = bytes
        .iter()
        .position(|byte| !byte.is_ascii())
        .unwrap_or(bytes.len());
    crate::standard::strtod(std::str::from_utf8(&bytes[..end]).unwrap()).0
}

/// `sscanf(line, "%Nc%Mc...")` into zeroed buffers: glibc copies each field until the
/// input ends, keeps a partially filled field, and leaves later fields empty.
pub(crate) fn scan_fields<'a>(line: &'a [u8], widths: &[usize]) -> Vec<&'a [u8]> {
    let mut position = 0;
    widths
        .iter()
        .map(|&width| {
            let start = position.min(line.len());
            position = (start + width).min(line.len());
            &line[start..position]
        })
        .collect()
}

/// A failed write into an injected output stream, as a Failure naming the cause.
pub(crate) fn write_failure(error: std::io::Error) -> Failure {
    Failure::new(&format!(
        "Units output: stream write failed ({error}); expected a writable stream"
    ))
}

/// Writes raw bytes (native `std::cout <<` of C strings) into an injected stream.
pub(crate) fn write_bytes(out: &mut dyn std::io::Write, bytes: &[u8]) -> Result<(), Failure> {
    out.write_all(bytes).map_err(write_failure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_string_helpers_follow_libc() {
        assert_eq!(c_str(b"ab\0cd"), b"ab");
        assert_eq!(strncmp(b"abc", b"abd", 2), Ordering::Equal);
        assert_eq!(strncmp(b"abc", b"abd", 3), Ordering::Less);
        assert_eq!(strncmp(b"\xb2", b"a", 1), Ordering::Greater);
        assert_eq!(strncmp(b"ab", b"abc", 3), Ordering::Less);
        assert_eq!(stream_real(0.001), "0.001");
        assert_eq!(right_adjust(b"ab \t\x0b"), b"ab");
        assert_eq!(c_string_to_real(b" 1.5e3x"), 1500.0);
        assert_eq!(
            scan_fields(b"abcde", &[2, 2, 2, 2]),
            [&b"ab"[..], b"cd", b"e", b""]
        );
    }

    #[test]
    fn sequence_errors_use_native_messages() {
        let values = new_sequence::<i32>();
        values.get().unwrap().append(4);
        assert_eq!(item(&values, 1).unwrap(), 4);
        let failure = item(&values, 2).err().unwrap();
        assert_eq!(failure.message(), "NCollection_Sequence::Value");
        let failure = edit(&values, REMOVE_ONE, |s| s.remove(3)).err().unwrap();
        assert_eq!(failure.message(), REMOVE_ONE);
        assert_eq!(length(&values).unwrap(), 1);
        assert!(object(&Handle::<HSequence<i32>>::default(), "X").is_err());
    }
}
