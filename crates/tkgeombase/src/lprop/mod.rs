// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/LProp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! LProp: the shared vocabulary of the local properties of curves and surfaces.
//!
//! The package holds the property status ([`Status`]), the kinds of particular
//! points of a curve ([`CIType`]) and their sorted store ([`CurAndInf`]), the two
//! native exceptions (as [`Failure`] constructors), and `LProp_CurveUtils`, the
//! algorithms of the curve property classes ([`curve_utils`]). `LProp_CLProps3d` is
//! the curve property class on an `Adaptor3d_Curve` handle ([`CLProps3d`]).
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Vector3;

mod cur_and_inf;
pub mod curve_access;
pub mod curve_utils;
pub use cur_and_inf::CurAndInf;

/// `LProp_Status`: the state of a lazily computed local property.
///
/// The order matters: the property classes test `status >= Defined`;
/// `Status::default() == Status::Undecided`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    /// Not computed yet.
    #[default]
    Undecided,
    /// Computed, and found undefined.
    Undefined,
    /// Known to be defined.
    Defined,
    /// Defined and stored.
    Computed,
}

/// `LProp_CIType`: the kind of a particular point of a curve;
/// `CIType::Inflection as i32 == 0`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CIType {
    /// A point of inflection.
    Inflection,
    /// A minimum of curvature.
    MinCur,
    /// A maximum of curvature.
    MaxCur,
}

/// `LProp_NotDefined`: the requested property does not exist at this point;
/// `not_defined("CLProps::Tangent: tangent undefined")`.
pub fn not_defined(message: &str) -> Failure {
    Failure::with_type("LProp_NotDefined", message, "")
}

/// `LProp_BadContinuity`: the curve is not continuous enough for the property.
///
/// No port path raises it (nor does the native package since the continuity test
/// left CLProps); it keeps the native type name;
/// `bad_continuity("CLProps: continuity C0; expected at least C2")`.
pub fn bad_continuity(message: &str) -> Failure {
    Failure::with_type("LProp_BadContinuity", message, "")
}

/// `LProp_CLProps3d`: the curve local properties of an `Adaptor3d_Curve` handle;
/// `CLProps3d::with_parameter(adaptor, 0.5, 2, 1e-7)?.curvature()?`.
pub type CLProps3d = crate::geom_lprop::CLPropsBase<Vector3, Handle<dyn Adaptor3dCurve>>;
