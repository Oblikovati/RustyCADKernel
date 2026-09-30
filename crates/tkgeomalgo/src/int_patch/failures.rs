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
// Rust translation of the exceptions raised by ModelingAlgorithms/TKGeomAlgo/IntPatch at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native exceptions of IntPatch as [`Failure`] values that name the member and the
//! offending value, and the 1-based sequence edits the lines make on their vertices.
use crate::int_start::failures::sequence_value;
use rustycad_tkernel::standard::Failure;

/// The Standard_DomainError of `member`; `domain_error("IntPatch_Point::ArcOnS1", "...")`.
pub(crate) fn domain_error(member: &str, reason: &str) -> Failure {
    Failure::with_message("Standard_DomainError", &format!("{member}() - {reason}"))
}

/// The Standard_NullObject of a member called on a null handle.
pub(crate) fn null_object(member: &str, what: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("{member}() - the {what} handle is null; expected an object"),
    )
}

/// The 0-based slot of the 1-based `index`, or the Standard_OutOfRange of
/// `NCollection_Sequence` outside `1..=len`.
pub(crate) fn sequence_slot<X>(items: &[X], index: i32, member: &str) -> Result<usize, Failure> {
    sequence_value(items, index, member)?;
    Ok(usize::try_from(index - 1).unwrap_or_default())
}

/// `NCollection_Sequence::ChangeValue(Index)`.
pub(crate) fn sequence_value_mut<'a, X>(
    items: &'a mut [X],
    index: i32,
    member: &str,
) -> Result<&'a mut X, Failure> {
    let slot: usize = sequence_slot(items, index, member)?;
    Ok(&mut items[slot])
}

/// `NCollection_Sequence::Remove(Index)`.
pub(crate) fn sequence_remove<X>(
    items: &mut Vec<X>,
    index: i32,
    member: &str,
) -> Result<X, Failure> {
    let slot: usize = sequence_slot(items, index, member)?;
    Ok(items.remove(slot))
}

/// `NCollection_Sequence::Exchange(I, J)`.
pub(crate) fn sequence_exchange<X>(
    items: &mut [X],
    first: i32,
    second: i32,
    member: &str,
) -> Result<(), Failure> {
    let a: usize = sequence_slot(items, first, member)?;
    let b: usize = sequence_slot(items, second, member)?;
    items.swap(a, b);
    Ok(())
}

/// `NCollection_Sequence::InsertBefore(Index, Item)`: Standard_OutOfRange outside
/// `1..=len`.
pub(crate) fn sequence_insert_before<X>(
    items: &mut Vec<X>,
    index: i32,
    item: X,
    member: &str,
) -> Result<(), Failure> {
    let slot: usize = sequence_slot(items, index, member)?;
    items.insert(slot, item);
    Ok(())
}
