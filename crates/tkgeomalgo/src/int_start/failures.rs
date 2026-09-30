// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the exceptions raised by ModelingAlgorithms/TKGeomAlgo/IntStart
// and IntPolyh at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native exceptions of IntStart and IntPolyh as [`Failure`] values that name the
//! member and the offending value.
use rustycad_tkernel::standard::Failure;

/// The `StdFail_NotDone` of a result read before a search is performed;
/// `not_done("IntStart_SearchInside::NbPoints")`.
pub(crate) fn not_done(member: &str) -> Failure {
    Failure::with_type(
        "StdFail_NotDone",
        &format!("{member}() - the search is not done; expected a Perform"),
        "",
    )
}

/// `NCollection_Sequence::Value(Index)`: the item of the 1-based `index`, or the
/// Standard_OutOfRange of an index outside `1..=len`.
/// `sequence_value(&points, 1, "IntStart_SearchOnBoundaries::Point")?`.
pub(crate) fn sequence_value<'a, X>(
    items: &'a [X],
    index: i32,
    member: &str,
) -> Result<&'a X, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| items.get(i))
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "{member}() - index {index} is outside the range 1..={}",
                items.len()
            ))
        })
}
