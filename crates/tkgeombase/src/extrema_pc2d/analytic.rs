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
// Rust translation of the PerformWithEndpoints members of ExtremaPC2d_Line.hxx,
// ExtremaPC2d_Circle.hxx, ExtremaPC2d_Ellipse.hxx, ExtremaPC2d_Hyperbola.hxx and
// ExtremaPC2d_Parabola.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `PerformWithEndpoints` flow the five analytical evaluators share: validate,
//! handle a degenerate domain by its single point, compute the interior extrema, then
//! add the ends of a bounded domain.
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, SearchMode, Status, add_endpoint_extrema,
    found_status, is_invalid_input,
};
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::Point2;
use std::convert::Infallible;
use std::f64::consts::PI;

/// An analytical evaluator: an infallible curve with an optional domain.
pub(crate) trait AnalyticEvaluator: CurveEvaluator<Point2, Error = Infallible> {
    /// The domain fixed at construction, `None` for the whole curve.
    fn fixed_domain(&self) -> Option<Domain1D>;
    /// Whether a domain spanning whole periods of 2*PI is closed (circle, ellipse).
    const IS_PERIODIC: bool;
    /// The reusable result.
    fn result_slot(&mut self) -> &mut ExtremaResult;
    /// `Perform`: the interior extrema into the result.
    fn perform_interior(&mut self, point: Point2, tol: f64, mode: SearchMode);
}

/// The shared `PerformWithEndpoints`.
pub(crate) fn perform_with_endpoints<E: AnalyticEvaluator>(
    evaluator: &mut E,
    point: Point2,
    tol: f64,
    mode: SearchMode,
) -> &ExtremaResult {
    let domain: Option<Domain1D> = evaluator.fixed_domain();
    if is_invalid_input(tol, &point, domain.as_ref()) {
        let result: &mut ExtremaResult = evaluator.result_slot();
        result.clear();
        result.status = Status::InvalidInput;
        return result;
    }
    if let Some(domain) = domain.filter(|domain| domain.min == domain.max) {
        let mut result: ExtremaResult = std::mem::take(evaluator.result_slot());
        result.clear();
        add_ends(evaluator, &mut result, &point, &domain, (mode, false));
        result.status = found_status(&result);
        *evaluator.result_slot() = result;
        return evaluator.result_slot();
    }
    evaluator.perform_interior(point, tol, mode);
    let mut result: ExtremaResult = std::mem::take(evaluator.result_slot());
    if let Some(domain) = domain
        && (result.status == Status::OK || result.status == Status::NoSolution)
    {
        let is_closed: bool = E::IS_PERIODIC
            && super::is_closed_periodic_domain(&domain, 2.0 * PI, precision::angular());
        add_ends(evaluator, &mut result, &point, &domain, (mode, is_closed));
        if !result.extrema.is_empty() {
            result.status = Status::OK;
        }
    }
    *evaluator.result_slot() = result;
    evaluator.result_slot()
}

// `AddEndpointExtrema` on an infallible evaluator.
fn add_ends<E: AnalyticEvaluator>(
    evaluator: &E,
    result: &mut ExtremaResult,
    point: &Point2,
    domain: &Domain1D,
    (mode, is_closed): (SearchMode, bool),
) {
    let Ok(()) = add_endpoint_extrema(result, point, domain, evaluator, mode, is_closed);
}
