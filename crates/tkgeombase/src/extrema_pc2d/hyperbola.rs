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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_Hyperbola.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_Hyperbola`: the extrema on a hyperbola from a quartic in `v = e^u`.
use super::analytic::{self, AnalyticEvaluator};
use super::{CurveEvaluator, Domain1D, ExtremaResult, SearchMode, Status};
use crate::extrema_pc_core::conic_roots::{PolynomialRoots, add_polynomial_extrema};
use rustycad_tkmath::gp::{Hyperbola2, Point2};
use rustycad_tkmath::math_poly::{PolyResult, quartic};
use std::convert::Infallible;

/// Point-hyperbola extrema (`ExtremaPC2d_Hyperbola`): for `C(u) = (R cosh u, r sinh u)`
/// the substitution `v = e^u` gives
/// `(R^2 + r^2)/4 v^4 - (X R + Y r)/2 v^3 + (X R - Y r)/2 v - (R^2 + r^2)/4 = 0`; each
/// positive root of odd multiplicity is an extremum (up to four);
/// `Hyperbola::new(hyperbola).perform(point, 1e-9, SearchMode::MinMax)`.
pub struct Hyperbola {
    hyperbola: Hyperbola2,
    domain: Option<Domain1D>,
    result: ExtremaResult,
    cache: HyperbolaCache,
}

/// The cached geometry of `cacheGeometry`.
#[derive(Clone, Copy)]
struct HyperbolaCache {
    center: (f64, f64),
    x_dir: (f64, f64),
    y_dir: (f64, f64),
    major: f64,
    minor: f64,
    r2_plus_r2_over_4: f64,
}

impl HyperbolaCache {
    fn new(hyperbola: &Hyperbola2) -> Self {
        let center: Point2 = hyperbola.location();
        let axis = hyperbola.axis();
        let (x_dir, y_dir) = (axis.x_direction(), axis.y_direction());
        let (major, minor) = (hyperbola.major_radius(), hyperbola.minor_radius());
        Self {
            center: (center.x(), center.y()),
            x_dir: (x_dir.x(), x_dir.y()),
            y_dir: (y_dir.x(), y_dir.y()),
            major,
            minor,
            r2_plus_r2_over_4: (major * major + minor * minor) / 4.0,
        }
    }
}

impl Hyperbola {
    /// Evaluator on the whole hyperbola (`ExtremaPC2d_Hyperbola(theHyperbola)`);
    /// `Hyperbola::new(hyperbola)`.
    pub fn new(hyperbola: Hyperbola2) -> Self {
        Self {
            cache: HyperbolaCache::new(&hyperbola),
            hyperbola,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC2d_Hyperbola(theHyperbola, theDomain)`);
    /// `Hyperbola::with_domain(hyperbola, Domain1D::new(-1.0, 1.0))`.
    pub fn with_domain(hyperbola: Hyperbola2, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(hyperbola)
        }
    }
    /// The point `C + R cosh(u) X + r sinh(u) Y` from the cached geometry (`Value`);
    /// `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point2 {
        let c = &self.cache;
        let (r_cosh, r_sinh) = (c.major * u.cosh(), c.minor * u.sinh());
        Point2::new(
            c.center.0 + r_cosh * c.x_dir.0 + r_sinh * c.y_dir.0,
            c.center.1 + r_cosh * c.x_dir.1 + r_sinh * c.y_dir.1,
        )
    }
    /// True with a domain (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        self.domain.is_some()
    }
    /// The domain (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> Option<&Domain1D> {
        self.domain.as_ref()
    }
    /// The hyperbola (`Hyperbola`); `extrema.hyperbola()`.
    pub fn hyperbola(&self) -> &Hyperbola2 {
        &self.hyperbola
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform(&mut self, point: Point2, tol: f64, mode: SearchMode) -> &ExtremaResult {
        self.result.clear();
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.status = Status::InvalidInput;
            return &self.result;
        }
        let roots: PolyResult = self.solve_quartic(point);
        let mut result: ExtremaResult = std::mem::take(&mut self.result);
        add_polynomial_extrema(
            &mut result,
            &PolynomialRoots::exponential(&roots),
            (&point, self.domain.as_ref(), mode),
            |u| self.value(u),
        );
        self.result = result;
        &self.result
    }
    /// The interior extrema and the extremal ends of a bounded arc
    /// (`PerformWithEndpoints`); `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> &ExtremaResult {
        analytic::perform_with_endpoints(self, point, tol, mode)
    }
    // `solveQuartic`: the quartic in v = e^u from the local query coordinates.
    fn solve_quartic(&self, point: Point2) -> PolyResult {
        let c = &self.cache;
        let (dx, dy) = (point.x() - c.center.0, point.y() - c.center.1);
        let x: f64 = dx * c.x_dir.0 + dy * c.x_dir.1;
        let y: f64 = dx * c.y_dir.0 + dy * c.y_dir.1;
        let c2: f64 = -(x * c.major + y * c.minor) / 2.0;
        let c4: f64 = (x * c.major - y * c.minor) / 2.0;
        quartic(c.r2_plus_r2_over_4, c2, 0.0, c4, -c.r2_plus_r2_over_4)
    }
}

impl CurveEvaluator<Point2> for Hyperbola {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point2, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator for Hyperbola {
    const IS_PERIODIC: bool = false;
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
