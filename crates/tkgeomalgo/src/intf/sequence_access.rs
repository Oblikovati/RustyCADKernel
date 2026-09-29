// Copyright (c) 1991-1999 Matra Datavision
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
// Port-only helpers for the NCollection_Sequence accesses of the Intf classes at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native `int`-indexed and First/Last accesses of NCollection_Sequence, with the
//! exceptions they raise (Standard_OutOfRange, Standard_NoSuchObject).
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// The one-based `int` index of a sequence access; a negative index maps to 0, which
/// the sequence refuses with Standard_OutOfRange as natively.
pub(crate) fn index(value: i32) -> usize {
    usize::try_from(value).unwrap_or(0)
}

/// `sequence(Index)` / `Value(Index)`: raises Standard_OutOfRange outside 1..=Length.
pub(crate) fn item<T>(sequence: &Sequence<T>, position: i32) -> Result<&T, Failure> {
    sequence.value(index(position))
}

/// `ChangeValue(Index)`: raises Standard_OutOfRange outside 1..=Length.
pub(crate) fn item_mut<T>(sequence: &mut Sequence<T>, position: i32) -> Result<&mut T, Failure> {
    sequence.change_value(index(position))
}

/// `First()`: raises Standard_NoSuchObject on an empty sequence.
pub(crate) fn first<T>(sequence: &Sequence<T>) -> Result<&T, Failure> {
    sequence.first().ok_or_else(|| empty("First"))
}

/// `Last()`: raises Standard_NoSuchObject on an empty sequence.
pub(crate) fn last<T>(sequence: &Sequence<T>) -> Result<&T, Failure> {
    sequence.last().ok_or_else(|| empty("Last"))
}

fn empty(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("NCollection_Sequence::{member} on an empty sequence; expected at least one item"),
        "",
    )
}

/// Writes `text` where native code streams to `std::cout`; a stream error is a
/// Standard_Failure naming the member.
pub(crate) fn write_text(
    stream: &mut dyn fmt::Write,
    text: &str,
    member: &str,
) -> Result<(), Failure> {
    stream.write_str(text).map_err(|_| {
        Failure::with_type(
            "Standard_Failure",
            &format!("{member}: the output stream refused {} bytes", text.len()),
            "",
        )
    })
}
