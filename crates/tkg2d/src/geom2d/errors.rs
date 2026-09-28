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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_UndefinedDerivative.hxx, Geom2d_UndefinedValue.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Native exception types raised by Geom2d, as [`Failure`] values.
//!
//! `Geom2d_UndefinedDerivative` and `Geom2d_UndefinedValue` derive from
//! Standard_DomainError natively; the failure keeps the most derived name.
use rustycad_tkernel::standard::Failure;

/// `Geom2d_UndefinedDerivative`; `undefined_derivative("Geom2d_Line::EvalDN: N = 0; expected >= 1")`.
pub fn undefined_derivative(message: &str) -> Failure {
    Failure::with_type("Geom2d_UndefinedDerivative", message, "")
}
/// `Geom2d_UndefinedValue`; `undefined_value("Geom2d_OffsetCurve::EvalD0: ...")`.
pub fn undefined_value(message: &str) -> Failure {
    Failure::with_type("Geom2d_UndefinedValue", message, "")
}
/// `Standard_ConstructionError`; `construction("Geom2d_Circle: Radius = -1; expected >= 0")`.
pub fn construction(message: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", message, "")
}
/// `Standard_RangeError`; `range("IsCN: N = -1; expected >= 0")`.
pub fn range(message: &str) -> Failure {
    Failure::with_type("Standard_RangeError", message, "")
}
/// `Standard_OutOfRange`; `out_of_range("Pole: Index = 9; expected in [1, 4]")`.
pub fn out_of_range(message: &str) -> Failure {
    Failure::out_of_range(message)
}
/// `Standard_DomainError`; `domain("Segment: U2 = 0 < U1 = 1; expected U1 <= U2")`.
pub fn domain(message: &str) -> Failure {
    Failure::with_type("Standard_DomainError", message, "")
}
/// `Standard_NoSuchObject`; `no_such_object("Period: the curve is not periodic")`.
pub fn no_such_object(message: &str) -> Failure {
    Failure::with_type("Standard_NoSuchObject", message, "")
}
/// `Standard_DimensionError`; `dimension("Poles: length 3; expected 4")`.
pub fn dimension(message: &str) -> Failure {
    Failure::with_type("Standard_DimensionError", message, "")
}
/// `Standard_ProgramError`; `program("SetEvalRepresentation: null Representation")`.
pub fn program(message: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, "")
}
/// Convert an `NCollection` bound failure into the native Standard_OutOfRange;
/// `array.value(i).map_err(array_bounds)?`.
pub fn array_bounds(error: impl std::fmt::Display) -> Failure {
    Failure::out_of_range(&error.to_string())
}
