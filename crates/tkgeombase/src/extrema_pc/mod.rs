// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ExtremaPC package: the extrema of the distance between a 3D point and a curve.
//!
//! One evaluator per curve kind, as in [`crate::extrema_pc2d`]. A planar curve (every
//! conic, and a Bezier, BSpline or offset curve whose poles lie exactly in one plane)
//! is first solved by the 2D evaluator in its plane, at unchanged parameters; the 3D
//! algorithm runs only when that fails. [`Curve`] picks the evaluator of a
//! GeomAdaptor_Curve, a GeomAdaptor_TransformedCurve or a Geom curve.
//!
//! Every `perform` returns `Result`: the plane projections construct gp and Geom2d
//! objects whose native constructors can raise, and a native exception propagates out
//! of `Perform`;
//! `Circle::new(circle).perform(point, 1e-9, SearchMode::MinMax)?.nb_ext() == 2`.
mod adaptor_curve;
mod bezier_curve;
mod bspline_curve;
mod circle;
mod conic_frame;
mod curve;
mod ellipse;
mod grid;
mod hyperbola;
mod line;
mod parabola;
mod planar;
mod spline;

pub use adaptor_curve::{OffsetCurve, OtherCurve};
pub use bezier_curve::BezierCurve;
pub use bspline_curve::BSplineCurve;
pub use circle::Circle;
pub use curve::{Curve, EvaluatorVariant};
pub use ellipse::Ellipse;
pub use hyperbola::Hyperbola;
pub use line::Line;
pub use parabola::Parabola;
pub use planar::{
    convert_planar_result, line_plane, project_bezier, project_bspline, project_offset,
    to_search_mode_2d,
};

pub(crate) use crate::extrema_pc_core::analytic::{found_status, is_invalid_input};
use crate::extrema_pc_core::{self as core, PcPoint, Space3d};
pub use crate::extrema_pc_core::{
    CurveEvaluator, SearchMode, Status, THE_BEZIER_DEGREE_MULTIPLIER, THE_BEZIER_MIN_SAMPLES,
    THE_BSPLINE_FALLBACK_SAMPLES, THE_BSPLINE_SPAN_MULTIPLIER, THE_DEFAULT_TOLERANCE,
    THE_OTHER_CURVE_NB_SAMPLES, THE_PARAM_TOLERANCE, is_closed_periodic_domain,
    is_finite_parameter, is_valid_tolerance,
};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::Point3;
pub use rustycad_tkmath::math_utils::Domain1D;

/// One extremum (`ExtremaPC::ExtremumResult`).
pub type ExtremumResult = core::PcExtremum<Point3>;
/// The extrema of one computation (`ExtremaPC::Result`).
pub type ExtremaResult = core::PcResult<Point3>;
/// The normalized stationarity function (`ExtremaPC_DistanceFunction`).
pub type DistanceFunction<'a> = core::PcDistanceFunction<'a, Space3d>;
/// The numerical evaluator over a parameter partition (`ExtremaPC_GridEvaluator`).
pub type GridEvaluator = core::PcGridEvaluator<Space3d>;

/// Ratio of the parameter range for the neighbor point of the endpoint classification
/// (`THE_NEIGHBOR_STEP_RATIO`; 1e-6 in ExtremaPC2d).
pub const THE_NEIGHBOR_STEP_RATIO: f64 = 0.01;

/// True when all coordinates are finite OCCT values (`IsFinitePoint`);
/// `is_finite_point(&Point3::new(0.0, 1.0, 2.0))`.
pub fn is_finite_point(point: &Point3) -> bool {
    point.is_finite_point()
}

/// Adds the finite ends of `domain` that are local extrema (`AddEndpointExtrema`).
///
/// As [`crate::extrema_pc2d::add_endpoint_extrema`], with the 3D neighbor step:
/// `min(span * THE_NEIGHBOR_STEP_RATIO, span / 2)` for a finite domain, else
/// `max(THE_NEIGHBOR_STEP_RATIO, |u| * THE_NEIGHBOR_STEP_RATIO)`;
/// `add_endpoint_extrema(&mut result, &p, &domain, &evaluator, SearchMode::MinMax, false)?`.
pub fn add_endpoint_extrema<E: CurveEvaluator<Point3> + ?Sized>(
    result: &mut ExtremaResult,
    query: &Point3,
    domain: &Domain1D,
    evaluator: &E,
    mode: SearchMode,
    is_closed: bool,
) -> Result<(), E::Error> {
    core::add_endpoint_extrema_with(
        result,
        query,
        domain,
        evaluator,
        (mode, is_closed),
        neighbor_step,
    )
}

/// The 3D neighbor step of AddEndpointExtrema.
pub(crate) fn neighbor_step(parameter: f64, span: f64, has_finite_span: bool) -> f64 {
    if has_finite_span {
        return std_min(span * THE_NEIGHBOR_STEP_RATIO, span * 0.5);
    }
    std_max(
        THE_NEIGHBOR_STEP_RATIO,
        parameter.abs() * THE_NEIGHBOR_STEP_RATIO,
    )
}
