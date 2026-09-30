// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of the exceptions raised by the sweep classes of
// ModelingAlgorithms/TKGeomAlgo/GeomFill at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native exceptions of the GeomFill sweep classes as [`Failure`] values
//! (docs/geomalgo-design.md, section 14).
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};

/// A failure of the native type `kind` with `message`;
/// `sweep_failure("Standard_DomainError", "GeomFill_SectionLaw::ConstantSection")`.
pub(crate) fn sweep_failure(kind: &'static str, message: &str) -> Failure {
    Failure::with_type(kind, message, "")
}

/// The Standard_NotImplemented of a member the class does not provide;
/// `not_implemented("GeomFill_LocationLaw::D1")`.
pub(crate) fn not_implemented(member: &str) -> Failure {
    sweep_failure("Standard_NotImplemented", member)
}

/// The bare `Standard_OutOfRange` of a continuity the member does not accept (the
/// native code raises it without a message).
pub(crate) fn out_of_range(member: &str, value: &str) -> Failure {
    sweep_failure(
        "Standard_OutOfRange",
        &format!("{member}: {value} is out of the accepted range"),
    )
}

/// The Standard_ConstructionError of an invalid argument;
/// `construction_error("GeomFill_Fixed : Two parallel vectors !")`.
pub(crate) fn construction_error(message: &str) -> Failure {
    sweep_failure("Standard_ConstructionError", message)
}

/// The Standard_DomainError of a member called outside its domain.
pub(crate) fn domain_error(message: &str) -> Failure {
    sweep_failure("Standard_DomainError", message)
}

/// The StdFail_NotDone of a result read before the computation succeeded.
pub(crate) fn not_done(member: &str) -> Failure {
    sweep_failure(
        "StdFail_NotDone",
        &format!("{member}: the computation is not done; expected IsDone()"),
    )
}

/// The object a native null handle dereference would crash on: here a
/// Standard_NullObject naming the member and the handle.
pub(crate) fn null_object(member: &str, handle: &str) -> Failure {
    sweep_failure(
        "Standard_NullObject",
        &format!("{member}: {handle} is a null handle; expected a loaded object"),
    )
}

/// Borrows the object of `handle`, or the Standard_NullObject a native null
/// dereference stands for; `live(&curve, "GeomFill_Frenet::D0", "myTrimmed")?`.
pub(crate) fn live<'a, T: ?Sized + RuntimeTyped>(
    handle: &'a Handle<T>,
    member: &str,
    name: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| null_object(member, name))
}

/// A `math_Matrix(LowerRow, UpperRow, LowerCol, UpperCol)` whose allocation error
/// becomes the Standard_RangeError the native constructor raises;
/// `matrix_with_bounds(1, 3, 1, 3)?`.
pub(crate) fn matrix_with_bounds(
    lower_row: i32,
    upper_row: i32,
    lower_col: i32,
    upper_col: i32,
) -> Result<rustycad_tkmath::math::Matrix<'static>, Failure> {
    rustycad_tkmath::math::Matrix::with_bounds(lower_row, upper_row, lower_col, upper_col).map_err(
        |error| {
            sweep_failure(
                "Standard_RangeError",
                &format!(
                    "math_Matrix({lower_row}, {upper_row}, {lower_col}, {upper_col}): {error}"
                ),
            )
        },
    )
}
