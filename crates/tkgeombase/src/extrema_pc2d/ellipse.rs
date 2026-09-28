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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_Ellipse.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_Ellipse`: the stationary points of the distance to an ellipse from a
//! trigonometric equation.
use super::analytic::{self, AnalyticEvaluator};
use super::circle::lift;
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status,
    THE_PARAM_TOLERANCE, found_status, is_closed_periodic_domain,
};
use rustycad_tkernel::precision;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Ellipse2, Point2, Vector2};
use rustycad_tkmath::math_root::trigonometric;
use rustycad_tkmath::math_root::utils::compute_periodic_root_count;
use std::convert::Infallible;
use std::f64::consts::PI;

/// Point-ellipse extrema (`ExtremaPC2d_Ellipse`): the roots of
/// `(B^2 - A^2) cos(u) sin(u) - B Y cos(u) + A X sin(u) = 0` in the ellipse frame,
/// classified by the sign of that function between neighboring roots (up to four
/// extrema). A point at the center of a circular ellipse has infinitely many solutions;
/// `Ellipse::new(ellipse).perform(point, 1e-9, SearchMode::MinMax)`.
pub struct Ellipse {
    ellipse: Ellipse2,
    domain: Option<Domain1D>,
    result: ExtremaResult,
}

/// The ellipse radii and the local query coordinates of one computation.
#[derive(Clone, Copy)]
struct Frame {
    a: f64,
    b: f64,
    x: f64,
    y: f64,
}

impl Frame {
    // The stationarity lambda: the sign of the distance derivative at `u`.
    fn stationarity(&self, u: f64) -> f64 {
        let (cos, sin) = (u.cos(), u.sin());
        (self.b * self.b - self.a * self.a) * cos * sin - self.b * self.y * cos
            + self.a * self.x * sin
    }
}

impl Ellipse {
    /// Evaluator on the full ellipse (`ExtremaPC2d_Ellipse(theEllipse)`); `Ellipse::new(ellipse)`.
    pub fn new(ellipse: Ellipse2) -> Self {
        Self {
            ellipse,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC2d_Ellipse(theEllipse, theDomain)`);
    /// `Ellipse::with_domain(ellipse, Domain1D::new(0.0, PI))`.
    pub fn with_domain(ellipse: Ellipse2, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(ellipse)
        }
    }
    /// The point at `u` (`Value`, ElCLib::Value); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point2 {
        elclib::value(u, &self.ellipse)
    }
    /// True for an arc (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        self.domain.is_some()
    }
    /// The domain of an arc (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> Option<&Domain1D> {
        self.domain.as_ref()
    }
    /// The ellipse (`Ellipse`); `extrema.ellipse()`.
    pub fn ellipse(&self) -> &Ellipse2 {
        &self.ellipse
    }
    /// The interior extrema over the domain, `[0, 2PI]` without one (`Perform`);
    /// `extrema.perform(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform(&mut self, point: Point2, tol: f64, mode: SearchMode) -> &ExtremaResult {
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.clear();
            self.result.status = Status::InvalidInput;
            return &self.result;
        }
        if self.domain.is_some_and(|domain| domain.min == domain.max) {
            self.result.clear();
            self.result.status = Status::NoSolution;
            return &self.result;
        }
        let domain: Domain1D = self.domain.unwrap_or(Domain1D::new(0.0, 2.0 * PI));
        self.perform_core(point, &domain, tol, mode);
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
        analytic::perform_with_endpoints(self, point, tol, mode)
    }
    // `performCore`: solve, canonicalize, classify and lift the roots.
    fn perform_core(&mut self, point: Point2, domain: &Domain1D, tol: f64, mode: SearchMode) {
        self.result.clear();
        if super::is_invalid_input(tol, &point, Some(domain)) {
            self.result.status = Status::InvalidInput;
            return;
        }
        let Some(frame) = self.frame(point) else {
            return;
        };
        let (ko2, ko3, ko4) = (
            (frame.b * frame.b - frame.a * frame.a) / 2.0,
            -frame.b * frame.y,
            frame.a * frame.x,
        );
        // MathRoot::Trigonometric handles all special cases including Y ~= 0
        let roots = trigonometric([0.0, ko2, ko3, ko4, 0.0], 0.0, 2.0 * PI, 1.5e-12);
        if roots.infinite_roots {
            self.result.status = Status::InfiniteSolutions;
            let on_curve: Point2 = elclib::value(0.0, &self.ellipse);
            self.result.infinite_square_distance = point.square_distance(on_curve);
            return;
        }
        if !roots.is_done() {
            self.result.status = Status::NumericalError;
            return;
        }
        let canonical: Vec<f64> = canonical_roots(&roots.roots[..roots.nb_roots]);
        self.classify(point, domain, frame, &canonical, mode);
    }
    // The local frame; `None` after setting InfiniteSolutions for a point at the
    // center of a circular ellipse.
    fn frame(&mut self, point: Point2) -> Option<Frame> {
        let (a, b) = (self.ellipse.major_radius(), self.ellipse.minor_radius());
        let center_to_point: Vector2 = Vector2::between(self.ellipse.location(), point);
        if center_to_point.magnitude() <= gp::resolution() && a == b {
            // Point at center of a circle - infinite solutions
            self.result.status = Status::InfiniteSolutions;
            self.result.infinite_square_distance = a * a;
            return None;
        }
        let axis = self.ellipse.axis();
        let x: f64 = center_to_point.dot(Vector2::from(axis.x_direction()));
        let y: f64 = center_to_point.dot(Vector2::from(axis.y_direction()));
        Some(Frame { a, b, x, y })
    }
    // Classifies the canonical roots by the stationarity sign at the mid-points to
    // their neighbors (cyclically) and adds the extrema.
    fn classify(
        &mut self,
        point: Point2,
        domain: &Domain1D,
        frame: Frame,
        roots: &[f64],
        mode: SearchMode,
    ) {
        let is_closed_domain: bool =
            is_closed_periodic_domain(domain, 2.0 * PI, precision::angular());
        let count: usize = roots.len();
        for (i, &root) in roots.iter().enumerate() {
            let previous: f64 = if i == 0 {
                roots[count - 1] - 2.0 * PI
            } else {
                roots[i - 1]
            };
            let next: f64 = if i + 1 == count {
                roots[0] + 2.0 * PI
            } else {
                roots[i + 1]
            };
            let left: f64 = frame.stationarity((previous + root) * 0.5);
            let right: f64 = frame.stationarity((root + next) * 0.5);
            let is_minimum: bool = left < 0.0 && right > 0.0;
            let is_maximum: bool = left > 0.0 && right < 0.0;
            if (is_minimum || is_maximum)
                && !self.add_extremum(point, domain, (root, is_minimum), (mode, is_closed_domain))
            {
                self.result.clear();
                self.result.status = Status::NumericalError;
                return;
            }
        }
        self.result.status = found_status(&self.result);
    }
    // The addExtremum lambda: lift into the domain, filter by mode, and add one
    // extremum per period; false on an enumeration error.
    fn add_extremum(
        &mut self,
        point: Point2,
        domain: &Domain1D,
        (root, is_min): (f64, bool),
        (mode, is_closed_domain): (SearchMode, bool),
    ) -> bool {
        let u: f64 = lift(root, domain.min, THE_PARAM_TOLERANCE);
        if (mode == SearchMode::Min && !is_min) || (mode == SearchMode::Max && is_min) {
            return true;
        }
        let mut nb_representatives: usize = 0;
        if !compute_periodic_root_count(
            u,
            domain.max,
            2.0 * PI,
            THE_PARAM_TOLERANCE,
            is_closed_domain,
            &mut nb_representatives,
        ) {
            return false;
        }
        let mut previous: f64 = 0.0;
        for period_index in 0..nb_representatives {
            let parameter: f64 = u + period_index as f64 * (2.0 * PI);
            if period_index > 0 && parameter <= previous {
                return false;
            }
            previous = parameter;
            let on_curve: Point2 = elclib::value(parameter, &self.ellipse);
            let square_distance: f64 = point.square_distance(on_curve);
            let extremum =
                ExtremumResult::new(parameter, on_curve, square_distance, is_min, !is_min);
            self.result.extrema.push(extremum);
        }
        true
    }
}

/// The roots reduced to `[0, 2PI)` (a value within THE_PARAM_TOLERANCE of 2PI becomes
/// 0), deduplicated within THE_PARAM_TOLERANCE in order, then sorted.
pub(crate) fn canonical_roots(roots: &[f64]) -> Vec<f64> {
    let mut canonical: Vec<f64> = Vec::with_capacity(4);
    for &root in roots {
        let mut value: f64 = root % (2.0 * PI);
        if value < 0.0 {
            value += 2.0 * PI;
        }
        if value >= 2.0 * PI - THE_PARAM_TOLERANCE {
            value = 0.0;
        }
        if !canonical
            .iter()
            .any(|existing| (existing - value).abs() < THE_PARAM_TOLERANCE)
        {
            canonical.push(value);
        }
    }
    canonical.sort_by(f64::total_cmp);
    canonical
}

impl CurveEvaluator<Point2> for Ellipse {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point2, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator for Ellipse {
    const IS_PERIODIC: bool = true;
    fn fixed_domain(&self) -> Option<Domain1D> {
        self.domain
    }
    fn result_slot(&mut self) -> &mut ExtremaResult {
        &mut self.result
    }
    fn perform_interior(&mut self, point: Point2, tol: f64, mode: SearchMode) {
        let _ = self.perform(point, tol, mode);
    }
}
