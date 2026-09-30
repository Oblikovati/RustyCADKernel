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
// Rust support for the NCollection_Sequence members that ModelingAlgorithms/TKTopAlgo/MAT2d uses, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 1-based `int` indexing of `NCollection_Sequence` over the port's sequence
//! (whose indices are `usize`): a negative index raises Standard_OutOfRange as an
//! index past the end does.
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;

fn slot(index: i32) -> Result<usize, Failure> {
    usize::try_from(index).map_err(|_| {
        errors::out_of_range(&format!(
            "NCollection_Sequence: index {index}; expected at least 1"
        ))
    })
}

/// `Value(index)`, cloned.
pub(crate) fn at<T: Clone>(seq: &Sequence<T>, index: i32) -> Result<T, Failure> {
    Ok(seq.value(slot(index)?)?.clone())
}

/// `ChangeValue(index)`.
pub(crate) fn at_mut<T>(seq: &mut Sequence<T>, index: i32) -> Result<&mut T, Failure> {
    seq.change_value(slot(index)?)
}

/// `InsertAfter(index, value)`.
pub(crate) fn insert_after<T>(seq: &mut Sequence<T>, index: i32, value: T) -> Result<(), Failure> {
    seq.insert_after(slot(index)?, value)?;
    Ok(())
}

/// `InsertBefore(index, value)`.
pub(crate) fn insert_before<T>(seq: &mut Sequence<T>, index: i32, value: T) -> Result<(), Failure> {
    seq.insert_before(slot(index)?, value)?;
    Ok(())
}

/// `Remove(index)`.
pub(crate) fn remove<T>(seq: &mut Sequence<T>, index: i32) -> Result<(), Failure> {
    seq.remove(slot(index)?)
}

/// `First()`, cloned: Standard_NoSuchObject on an empty sequence.
pub(crate) fn first<T: Clone>(seq: &Sequence<T>) -> Result<T, Failure> {
    seq.first().cloned().ok_or_else(|| {
        errors::no_such_object("NCollection_Sequence::First: empty; expected an item")
    })
}

/// `Last()`, cloned: Standard_NoSuchObject on an empty sequence.
pub(crate) fn last<T: Clone>(seq: &Sequence<T>) -> Result<T, Failure> {
    seq.last().cloned().ok_or_else(|| {
        errors::no_such_object("NCollection_Sequence::Last: empty; expected an item")
    })
}
