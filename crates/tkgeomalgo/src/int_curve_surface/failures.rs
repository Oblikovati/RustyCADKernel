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
// Rust translation of the exceptions raised in ModelingAlgorithms/TKGeomAlgo/IntCurveSurface at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native exceptions of IntCurveSurface as [`Failure`] values that name the
//! member and the offending value.
use rustycad_tkernel::standard::Failure;

/// The `StdFail_NotDone` of a result read before a computation succeeded;
/// `not_done("IntCurveSurface_Intersection::NbPoints")`.
pub(crate) fn not_done(member: &str) -> Failure {
    Failure::with_type(
        "StdFail_NotDone",
        &format!("{member}() - the computation is not done; expected IsDone()"),
        "",
    )
}

/// The `Standard_OutOfRange` of an index outside `1..=len`;
/// `out_of_range("IntCurveSurface_Intersection::Point", 3, 2)`.
pub(crate) fn out_of_range(member: &str, index: i32, len: usize) -> Failure {
    Failure::out_of_range(&format!(
        "{member}() - index {index} is outside the range 1..={len}"
    ))
}

/// The `Standard_OutOfRange` of an index outside `lower..=upper`;
/// `out_of_bounds("IntCurveSurface_ThePolygonOfHInter::BeginOfSeg", 0, 1, 5)`.
pub(crate) fn out_of_bounds(member: &str, index: i32, lower: i32, upper: i32) -> Failure {
    Failure::out_of_range(&format!(
        "{member}() - index {index} is outside the range {lower}..={upper}"
    ))
}
