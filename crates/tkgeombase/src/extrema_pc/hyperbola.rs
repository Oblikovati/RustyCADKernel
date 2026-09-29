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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Hyperbola.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_Hyperbola`: the extrema on a hyperbola in space.
use super::conic_frame::ConicFrame;
use super::planar::{PlanarQuery, perform_planar};
use super::{CurveEvaluator, Domain1D, ExtremaResult, SearchMode, Status};
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use crate::extrema_pc_core::conic_roots::{PolynomialRoots, add_polynomial_extrema};
use crate::extrema_pc_core::{NeighborStep, ValueFn};
use crate::extrema_pc2d;
use crate::proj_lib as projlib;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis3, Hyperbola3, Plane3, Point3};
use rustycad_tkmath::math_poly::{PolyResult, quartic};
use std::convert::Infallible;

/// Point-hyperbola extrema (`ExtremaPC_Hyperbola`): solved by `ExtremaPC2d_Hyperbola`
/// in the hyperbola plane; when that fails, the quartic in `v = e^u` of the 2D evaluator
/// for the point projected on the plane;
/// `Hyperbola::new(hyperbola).perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct Hyperbola {
    hyperbola: Hyperbola3,
    domain: Option<Domain1D>,
    result: ExtremaResult,
    frame: ConicFrame,
    major: f64,
    minor: f64,
    r2_plus_r2_over_4: f64,
}

/// `C + R cosh(u) X + r sinh(u) Y` from the cached geometry.
fn hyperbola_value(frame: &ConicFrame, major: f64, minor: f64, u: f64) -> Point3 {
    frame.point(major * u.cosh(), minor * u.sinh())
}

impl Hyperbola {
    /// Evaluator on the whole hyperbola (`ExtremaPC_Hyperbola(theHyperbola)`);
    /// `Hyperbola::new(hyperbola)`.
    pub fn new(hyperbola: Hyperbola3) -> Self {
        let (major, minor) = (hyperbola.major_radius(), hyperbola.minor_radius());
        Self {
            frame: ConicFrame::of(hyperbola.position()),
            hyperbola,
            domain: None,
            result: ExtremaResult::default(),
            major,
            minor,
            r2_plus_r2_over_4: (major * major + minor * minor) / 4.0,
        }
    }
    /// Evaluator on `domain` (`ExtremaPC_Hyperbola(theHyperbola, theDomain)`);
    /// `Hyperbola::with_domain(hyperbola, Domain1D::new(-1.0, 1.0))`.
    pub fn with_domain(hyperbola: Hyperbola3, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(hyperbola)
        }
    }
    /// The point at `u` from the cached geometry (`Value`); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point3 {
        hyperbola_value(&self.frame, self.major, self.minor, u)
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
    pub fn hyperbola(&self) -> &Hyperbola3 {
        &self.hyperbola
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        let valid: bool = !super::is_invalid_input(tol, &point, self.domain.as_ref());
        if valid && self.perform_in_plane(point, tol, mode)? {
            return Ok(&self.result);
        }
        self.perform_core(point, tol, mode);
        Ok(&self.result)
    }
    /// The interior extrema and the extremal ends of a bounded arc
    /// (`PerformWithEndpoints`); `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        analytic::perform_with_endpoints(self, point, tol, mode)
    }
    // ExtremaPC2d_Hyperbola in the hyperbola plane.
    fn perform_in_plane(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<bool, Failure> {
        let plane = Plane3::from_position(Axis3::from_axis2(self.hyperbola.position()));
        let hyperbola2d = projlib::project_hyperbola_on_plane(&plane, &self.hyperbola)?;
        let query = PlanarQuery {
            point2d: projlib::project_point_on_plane(&plane, point),
            point3d: point,
            plane: &plane,
            tol,
            mode,
            with_endpoints: false,
        };
        let (frame, major, minor) = (self.frame, self.major, self.minor);
        let evaluator =
            ValueFn(|u| Ok::<Point3, Infallible>(hyperbola_value(&frame, major, minor, u)));
        let Ok(done) = perform_planar::<extrema_pc2d::Hyperbola, _>(
            hyperbola2d,
            self.domain,
            &query,
            &evaluator,
            &mut self.result,
        );
        Ok(done)
    }
    // `performCore`: the quartic of the point projected on the plane.
    fn perform_core(&mut self, point: Point3, tol: f64, mode: SearchMode) {
        self.result.clear();
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.status = Status::InvalidInput;
            return;
        }
        let (x, y) = self.frame.local_coordinates(point);
        let c2: f64 = -(x * self.major + y * self.minor) / 2.0;
        let c4: f64 = (x * self.major - y * self.minor) / 2.0;
        let roots: PolyResult =
            quartic(self.r2_plus_r2_over_4, c2, 0.0, c4, -self.r2_plus_r2_over_4);
        let mut result: ExtremaResult = std::mem::take(&mut self.result);
        add_polynomial_extrema(
            &mut result,
            &PolynomialRoots::exponential(&roots),
            (&point, self.domain.as_ref(), mode),
            |u| self.value(u),
        );
        self.result = result;
    }
}

impl CurveEvaluator<Point3> for Hyperbola {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point3, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point3> for Hyperbola {
    const IS_PERIODIC: bool = false;
    const NEIGHBOR_STEP: NeighborStep = super::neighbor_step;
    type PerformError = Failure;
    fn fixed_domain(&self) -> Option<Domain1D> {
        self.domain
    }
    fn result_slot(&mut self) -> &mut ExtremaResult {
        &mut self.result
    }
    fn perform_interior(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<(), Failure> {
        self.perform(point, tol, mode).map(|_| ())
    }
}
