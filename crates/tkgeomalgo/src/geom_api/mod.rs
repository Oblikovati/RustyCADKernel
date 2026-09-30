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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomAPI at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package GeomAPI: the 3D geometry algorithms of the geometry API (the
//! plane conversions of the `GeomAPI` class, point projections, extrema,
//! curve-surface and surface-surface intersections, interpolation and approximation of
//! points).
//!
//! The static members of the `GeomAPI` class are the free functions [`to_2d`] and
//! [`to_3d`]. As in Geom2dAPI, each constructor overload is a named constructor
//! and each `Init` overload an `init_*` method, output references become returned
//! tuples, the `operator int`/`operator double`/`operator gp_Pnt`/`operator
//! occ::handle<...>` conversions are the `as_int`, `as_real`, `as_point` and
//! `as_curve`/`as_surface` methods, and native exceptions are returned as `Err`.
//! `GeomAPI_IntSS` ([`IntSS`]) wraps `GeomInt_IntSS` (`crate::geom_int`).
mod extrema_curve_curve;
mod extrema_curve_curve_total;
mod extrema_curve_surface;
mod extrema_surface_surface;
mod int_cs;
mod int_ss;
mod interpolate;
mod plane_curves;
mod points_to_bspline;
mod points_to_bspline_surface;
mod points_to_bspline_surface_fit;
mod points_to_bspline_surface_tangent;
mod points_to_bspline_surface_z;
mod project_point_on_curve;
mod project_point_on_surf;

pub use extrema_curve_curve::ExtremaCurveCurve;
pub use extrema_curve_surface::ExtremaCurveSurface;
pub use extrema_surface_surface::ExtremaSurfaceSurface;
pub use int_cs::IntCS;
pub use int_ss::IntSS;
pub use interpolate::{Interpolate, Interpolate3dSpace};
pub use plane_curves::{to_2d, to_3d};
pub use points_to_bspline::PointsToBSpline;
pub use points_to_bspline_surface::{HeightGrid, PointsToBSplineSurface, SurfaceFitSettings};
pub use project_point_on_curve::ProjectPointOnCurve;
pub use project_point_on_surf::ProjectPointOnSurf;

use crate::geom2d_api::{check_index, not_done};
use rustycad_tkernel::standard::Failure;

/// The StdFail_NotDone of a nearest-extremum member when the computation failed.
fn check_done(is_done: bool, member: &str) -> Result<(), Failure> {
    if !is_done {
        return Err(not_done(member));
    }
    Ok(())
}
