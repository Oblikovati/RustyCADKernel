// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
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
// Rust translation of ModelingData/TKGeomBase/GeomLib at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomLib: basic computations on Geom and Geom2d entities (placing 2D curves in
//! 3D, general transformations, reparametrization, 3D curves of curves on
//! surfaces, extensions, inertia axes, parameter arrays, closedness and normal
//! estimation), and the classes of the package.
//!
//! The static members of the native `GeomLib` class are free functions
//! (`GeomLib::To3d` is [`to_3d`]); each class is a struct. Output reference
//! parameters that the native code writes unconditionally are returned values;
//! in-out handles (`occ::handle<Geom_BoundedCurve>& Curve`, `NewCurvePtr`) stay
//! `&mut` parameters, because the native members may leave them untouched.
//!
//! `let curve_3d = geom_lib::to_3d(frame, &curve_2d)?;`
mod adjust_extremity;
mod build_curve3d;
mod cancel_denominator;
mod check_2d_bspline_curve;
mod check_bspline_curve;
mod check_curve_on_surface;
mod closedness;
mod compute_lambda;
mod denominator_multiplier;
mod eval_distance;
mod extend_curve;
mod extend_surface;
mod extend_surface_poles;
mod gtransform;
mod inertia;
mod interpolate;
mod is_planar_surface;
mod iso_line;
mod log_sample;
mod make_curve_from_approx;
mod norm_estim;
mod parameter_arrays;
mod poly_func;
mod same_range;
mod to_3d;
mod tool;
mod tool_deviation;

pub use adjust_extremity::adjust_extremity;
pub use build_curve3d::build_curve3d;
pub use cancel_denominator::cancel_denominator_derivative;
pub use check_2d_bspline_curve::Check2dBSplineCurve;
pub use check_bspline_curve::CheckBSplineCurve;
pub use check_curve_on_surface::CheckCurveOnSurface;
pub use closedness::{
    is_bspl_u_closed, is_bspl_v_closed, is_bz_u_closed, is_bz_v_closed, is_closed,
};
pub use denominator_multiplier::DenominatorMultiplier;
pub use eval_distance::{eval_max_distance_along_parameter, eval_max_parametric_distance};
pub use extend_curve::extend_curve_to_point;
pub use extend_surface::extend_surf_by_length;
pub use gtransform::gtransform;
pub use inertia::{AxeOfInertia, InertiaAxes, axe_of_inertia, inertia};
pub use interpolate::{Interpolate, InterpolationErrors};
pub use is_planar_surface::IsPlanarSurface;
pub use iso_line::{IsoLine, build_c3d_on_iso_line, is_iso_line};
pub use log_sample::LogSample;
pub use make_curve_from_approx::MakeCurvefromApprox;
pub use norm_estim::norm_estim;
pub use parameter_arrays::{densify_array1_of_real, fuse_intervals, remove_points_from_array};
pub use poly_func::PolyFunc;
pub use same_range::same_range;
pub use to_3d::to_3d;
pub use tool::{DeviationResult, Tool, compute_deviation, compute_deviation_pso};

use crate::native_failure::failure;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};

/// The object of a non-null handle; Standard_NullObject (naming `member`) where the
/// native code dereferences a null handle.
pub(crate) fn live<'h, T: ?Sized + RuntimeTyped>(
    handle: &'h Handle<T>,
    member: &str,
) -> Result<&'h T, Failure> {
    handle.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            &format!("{member}: null handle; expected an object"),
        )
    })
}
