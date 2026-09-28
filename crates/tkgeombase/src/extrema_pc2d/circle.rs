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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_Circle.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_Circle`: the nearest and farthest points of a circle.
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status,
    THE_PARAM_TOLERANCE, found_status, is_closed_periodic_domain,
};
use crate::extrema_pc_core::NeighborStep;
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use rustycad_tkernel::precision;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis22d, Circle2, Point2, Vector2};
use rustycad_tkmath::math_root::utils::compute_periodic_root_count;
use std::convert::Infallible;
use std::f64::consts::PI;

/// Point-circle extrema (`ExtremaPC2d_Circle`): the angle of the center-to-point vector
/// is the minimum and the opposite angle the maximum. A point at the center (or a
/// circle of zero radius) has infinitely many solutions. With a domain each solution
/// is lifted into it and repeated once per period it contains;
/// `Circle::new(circle).perform(point, 1e-9, SearchMode::MinMax).nb_ext() == 2`.
pub struct Circle {
    circle: Circle2,
    domain: Option<Domain1D>,
    result: ExtremaResult,
}

impl Circle {
    /// Evaluator on the full circle (`ExtremaPC2d_Circle(theCircle)`); `Circle::new(circle)`.
    pub fn new(circle: Circle2) -> Self {
        Self {
            circle,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC2d_Circle(theCircle, theDomain)`);
    /// `Circle::with_domain(circle, Domain1D::new(0.0, PI))`.
    pub fn with_domain(circle: Circle2, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(circle)
        }
    }
    /// The point at `u` (`Value`, ElCLib::Value); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point2 {
        elclib::value(u, &self.circle)
    }
    /// True for an arc (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        self.domain.is_some()
    }
    /// The domain of an arc (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> Option<&Domain1D> {
        self.domain.as_ref()
    }
    /// The circle (`Circle`); `extrema.circle()`.
    pub fn circle(&self) -> &Circle2 {
        &self.circle
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform(&mut self, point: Point2, tol: f64, mode: SearchMode) -> &ExtremaResult {
        self.result.clear();
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.status = Status::InvalidInput;
            return &self.result;
        }
        if self.domain.is_some_and(|domain| domain.min == domain.max) {
            self.result.status = Status::NoSolution;
            return &self.result;
        }
        if self.circle.radius() <= gp::resolution() {
            self.result.status = Status::InfiniteSolutions;
            self.result.infinite_square_distance = point.square_distance(self.circle.location());
            return &self.result;
        }
        let center_to_point: Vector2 = Vector2::between(self.circle.location(), point);
        if center_to_point.magnitude() <= gp::resolution() {
            self.result.status = Status::InfiniteSolutions;
            let radius: f64 = self.circle.radius();
            self.result.infinite_square_distance = radius * radius;
            return &self.result;
        }
        let solutions: [f64; 2] = self.canonical_solutions(center_to_point);
        self.add_solutions(point, solutions, mode);
        &self.result
    }
    /// The interior extrema and the extremal ends of an arc (`PerformWithEndpoints`);
    /// `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> &ExtremaResult {
        let Ok(result) = analytic::perform_with_endpoints(self, point, tol, mode);
        result
    }
    // The minimum angle (snapped to +-PI within Angular), its opposite, both lifted
    // into the domain.
    fn canonical_solutions(&self, center_to_point: Vector2) -> [f64; 2] {
        let position: Axis22d = self.circle.position();
        let mut us1: f64 = center_to_point
            .dot(Vector2::from(position.y_direction()))
            .atan2(center_to_point.dot(Vector2::from(position.x_direction())));
        let ang_tol: f64 = precision::angular();
        if us1 + PI < ang_tol {
            us1 = -PI;
        } else if us1 - PI > -ang_tol {
            us1 = PI;
        }
        // Us2 = Us1 + PI corresponds to maximum distance (farthest point)
        let mut us2: f64 = us1 + PI;
        if let Some(domain) = self.domain {
            let lift_tolerance: f64 = precision::angular();
            us1 = lift(us1, domain.min, lift_tolerance);
            us2 = lift(us2, domain.min, lift_tolerance);
        }
        [us1, us2]
    }
    // Appends the minimum (index 0) and the maximum (index 1) in the search mode, each
    // once per period of the domain.
    fn add_solutions(&mut self, point: Point2, solutions: [f64; 2], mode: SearchMode) {
        let start: usize = if mode == SearchMode::Max { 1 } else { 0 };
        let end: usize = if mode == SearchMode::Min { 1 } else { 2 };
        let is_closed_domain: bool = self.domain.is_some_and(|domain| {
            is_closed_periodic_domain(&domain, 2.0 * PI, precision::angular())
        });
        for (index, &solution) in solutions.iter().enumerate().take(end).skip(start) {
            let mut nb_representatives: usize = 1;
            if let Some(domain) = self.domain {
                let counted: bool = compute_periodic_root_count(
                    solution,
                    domain.max,
                    2.0 * PI,
                    THE_PARAM_TOLERANCE,
                    is_closed_domain,
                    &mut nb_representatives,
                );
                if !counted {
                    self.fail();
                    return;
                }
            }
            if !self.add_representatives(point, solution, nb_representatives, index == 0) {
                self.fail();
                return;
            }
        }
        self.result.status = found_status(&self.result);
    }
    // One extremum per period; false when the periodic parameters stop increasing.
    fn add_representatives(
        &mut self,
        point: Point2,
        solution: f64,
        count: usize,
        is_min: bool,
    ) -> bool {
        let mut previous: f64 = 0.0;
        for period_index in 0..count {
            let u: f64 = solution + period_index as f64 * (2.0 * PI);
            if period_index > 0 && u <= previous {
                return false;
            }
            previous = u;
            let on_curve: Point2 = elclib::value(u, &self.circle);
            let square_distance: f64 = point.square_distance(on_curve);
            self.result.extrema.push(ExtremumResult::new(
                u,
                on_curve,
                square_distance,
                is_min,
                !is_min,
            ));
        }
        true
    }
    // `myResult.Clear(); Status = NumericalError`.
    fn fail(&mut self) {
        self.result.clear();
        self.result.status = Status::NumericalError;
    }
}

/// `U += ceil((UMin - U - Tol) / 2PI) * 2PI`, snapped to UMin within Tol: the first
/// representative of `u` not below the domain start.
pub(crate) fn lift(u: f64, u_min: f64, tolerance: f64) -> f64 {
    let lifted: f64 = u + ((u_min - u - tolerance) / (2.0 * PI)).ceil() * (2.0 * PI);
    if (lifted - u_min).abs() <= tolerance {
        return u_min;
    }
    lifted
}

impl CurveEvaluator<Point2> for Circle {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point2, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point2> for Circle {
    const IS_PERIODIC: bool = true;
    const NEIGHBOR_STEP: NeighborStep = super::neighbor_step;
    type PerformError = Infallible;
    fn fixed_domain(&self) -> Option<Domain1D> {
        self.domain
    }
    fn result_slot(&mut self) -> &mut ExtremaResult {
        &mut self.result
    }
    fn perform_interior(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<(), Infallible> {
        let _ = self.perform(point, tol, mode);
        Ok(())
    }
}
