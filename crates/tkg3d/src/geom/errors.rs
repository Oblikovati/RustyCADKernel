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
// Rust translation of ModelingData/TKG3d/Geom/Geom_UndefinedDerivative.hxx, Geom_UndefinedValue.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Native exception types raised by Geom, as [`Failure`] values.
//!
//! `Geom_UndefinedDerivative` and `Geom_UndefinedValue` derive from
//! Standard_DomainError natively; the failure keeps the most derived name. The
//! Standard exceptions are shared with Geom2d.
use rustycad_tkernel::standard::Failure;
pub use rustycad_tkg2d::geom2d::errors::{
    array_bounds, construction, dimension, domain, no_such_object, out_of_range, program, range,
};

/// `Geom_UndefinedDerivative`; `undefined_derivative("Geom_Line::EvalDN: N = 0; expected >= 1")`.
pub fn undefined_derivative(message: &str) -> Failure {
    Failure::with_type("Geom_UndefinedDerivative", message, "")
}
/// `Geom_UndefinedValue`; `undefined_value("Geom_OffsetCurve::EvalD0: ...")`.
pub fn undefined_value(message: &str) -> Failure {
    Failure::with_type("Geom_UndefinedValue", message, "")
}
/// `Standard_NotImplemented`; `not_implemented("Geom_SurfaceOfRevolution::...")`.
pub fn not_implemented(message: &str) -> Failure {
    Failure::with_type("Standard_NotImplemented", message, "")
}
