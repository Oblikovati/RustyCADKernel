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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Geom2dAPI at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Geom2dAPI: the 2D curve algorithms of the geometry API
//! (extrema, intersection, projection, interpolation and approximation of points).
//!
//! The `operator int`/`operator double`/`operator gp_Pnt2d`/`operator
//! occ::handle<...>` conversions are the `as_int`, `as_real`, `as_point` and
//! `as_curve` methods.
mod extrema_curve_curve;
pub(crate) mod extrema_results;
mod inter_curve_curve;
mod interpolate;
mod interpolate_build;
mod interpolate_perform;
pub mod interpolate_space;
pub(crate) mod points_to_bspline;
pub(crate) mod points_to_bspline_y;
mod project_point_on_curve;

pub use extrema_curve_curve::ExtremaCurveCurve;
pub use inter_curve_curve::{CurvePair, InterCurveCurve};
pub use interpolate::{Interpolate, InterpolateGen};
pub use points_to_bspline::PointsToBSpline;
pub use project_point_on_curve::ProjectPointOnCurve;

use rustycad_tkernel::standard::Failure;

/// The StdFail_NotDone of a result read before the computation succeeded.
pub(crate) fn not_done(member: &str) -> Failure {
    Failure::with_type(
        "StdFail_NotDone",
        &format!("{member}() - the computation is not done; expected IsDone()"),
        "",
    )
}

/// The Standard_OutOfRange of an index outside `1..=count`.
pub(crate) fn check_index(index: i32, count: i32, member: &str) -> Result<(), Failure> {
    if index < 1 || index > count {
        return Err(Failure::out_of_range(&format!(
            "{member}() - index {index} outside [1, {count}]"
        )));
    }
    Ok(())
}
