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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ExtremaPC2d package: the extrema of the distance between a 2D point and a
//! curve.
//!
//! One evaluator per curve kind: analytical solutions for the line, circle, ellipse,
//! hyperbola and parabola, and a derivative-aware numerical root isolation over a
//! parameter partition ([`GridEvaluator`]) for Bezier, BSpline, offset and general
//! curves. [`Curve`] picks the evaluator of an adaptor or a Geom2d curve. Each
//! evaluator keeps its result and hands out a reference to it, as the native
//! `const Result& Perform(...) const` does (here through `&mut self`).
//!
//! The result types, the distance function and the grid evaluator are the generic
//! code of [`crate::extrema_pc_core`], instantiated for `gp_Pnt2d`;
//! `let mut extrema = Circle::new(circle); extrema.perform(point, 1e-9, SearchMode::MinMax)`.
mod adaptor_curve;
mod bezier_curve;
mod bspline_curve;
mod circle;
mod curve;
mod ellipse;
mod grid;
mod hyperbola;
mod line;
mod parabola;

pub use adaptor_curve::{OffsetCurve, OtherCurve};
pub use bezier_curve::BezierCurve;
pub use bspline_curve::BSplineCurve;
pub use circle::Circle;
pub(crate) use circle::lift;
pub use curve::{Curve, EvaluatorVariant};
pub use ellipse::Ellipse;
pub(crate) use ellipse::{EllipseFrame, canonical_roots};
pub use hyperbola::Hyperbola;
pub use line::Line;
pub(crate) use line::std_clamp;
pub use parabola::Parabola;

pub(crate) use crate::extrema_pc_core::analytic::{found_status, is_invalid_input};
use crate::extrema_pc_core::{self as core, PcPoint, Space2d};
pub use crate::extrema_pc_core::{
    CurveEvaluator, SearchMode, Status, THE_BEZIER_DEGREE_MULTIPLIER, THE_BEZIER_MIN_SAMPLES,
    THE_BSPLINE_FALLBACK_SAMPLES, THE_BSPLINE_SPAN_MULTIPLIER, THE_DEFAULT_TOLERANCE,
    THE_OTHER_CURVE_NB_SAMPLES, THE_PARAM_TOLERANCE, is_closed_periodic_domain,
    is_finite_parameter, is_valid_tolerance,
};
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::Point2;
pub use rustycad_tkmath::math_utils::Domain1D;

/// One extremum (`ExtremaPC2d::ExtremumResult`).
pub type ExtremumResult = core::PcExtremum<Point2>;
/// The extrema of one computation (`ExtremaPC2d::Result`).
pub type ExtremaResult = core::PcResult<Point2>;
/// The normalized stationarity function (`ExtremaPC2d_DistanceFunction`).
pub type DistanceFunction<'a> = core::PcDistanceFunction<'a, Space2d>;
/// The numerical evaluator over a parameter partition (`ExtremaPC2d_GridEvaluator`).
pub type GridEvaluator = core::PcGridEvaluator<Space2d>;

/// Maximum ratio of the parameter range for the one-sided endpoint classification
/// (`THE_NEIGHBOR_STEP_RATIO`; 0.01 in ExtremaPC).
pub const THE_NEIGHBOR_STEP_RATIO: f64 = 1.0e-6;

/// True when both coordinates are finite OCCT values (`IsFinitePoint`);
/// `is_finite_point(&Point2::new(0.0, 1.0))`.
pub fn is_finite_point(point: &Point2) -> bool {
    point.is_finite_point()
}

/// Adds the finite ends of `domain` that are local extrema (`AddEndpointExtrema`).
///
/// An end is a minimum when its inner neighbor is not nearer, a maximum when it is not
/// farther; the neighbor step is `min(span / 2, max(span * THE_NEIGHBOR_STEP_RATIO,
/// PConfusion))` (PConfusion for a half-infinite domain). A degenerate domain adds its
/// single point, a closed one nothing, and ends already present are skipped;
/// `add_endpoint_extrema(&mut result, &p, &domain, &evaluator, SearchMode::MinMax, false)?`.
pub fn add_endpoint_extrema<E: CurveEvaluator<Point2> + ?Sized>(
    result: &mut ExtremaResult,
    query: &Point2,
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

/// The 2D neighbor step of AddEndpointExtrema.
pub(crate) fn neighbor_step(_parameter: f64, span: f64, has_finite_span: bool) -> f64 {
    let resolution: f64 = precision::p_confusion();
    if has_finite_span {
        return std_min(
            span * 0.5,
            std_max(span * THE_NEIGHBOR_STEP_RATIO, resolution),
        );
    }
    resolution
}
