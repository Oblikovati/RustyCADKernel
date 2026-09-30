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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_TgtField.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_TgtField`: the abstract field of tangents along a boundary.
use crate::law::LawBSpline;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkmath::gp::Vector3;

/// A field of tangents along a boundary of a constrained filling
/// (`GeomFill_TgtField`), reached through `Handle<dyn GeomFillTgtField>`. The
/// evaluators return `Result`, because the boundaries behind a field raise natively;
/// the two `D1` overloads are `d1` and `d1_with_value`.
/// `let (t, dt) = field.d1_with_value(0.5)?;`
pub trait GeomFillTgtField: Transient {
    /// `IsScalable()`: false unless overridden.
    fn is_scalable(&self) -> bool {
        false
    }
    /// `Scale(Func)`: does nothing unless overridden.
    fn scale(&self, _func: &Handle<LawBSpline>) -> Result<(), Failure> {
        Ok(())
    }
    /// `Value(W)`: the tangent at `w`.
    fn value(&self, w: f64) -> Result<Vector3, Failure>;
    /// `D1(W)`: the derivative of the tangent at `w`.
    fn d1(&self, w: f64) -> Result<Vector3, Failure>;
    /// `D1(W, V, DV)`: the tangent and its derivative at `w`.
    fn d1_with_value(&self, w: f64) -> Result<(Vector3, Vector3), Failure>;
}
impl_transient_interface!(dyn GeomFillTgtField, "GeomFill_TgtField", StandardTransient);
