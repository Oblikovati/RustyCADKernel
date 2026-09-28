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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Ellipse.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_Ellipse`: the extrema on an ellipse in space.
use super::planar::{PlanarQuery, perform_planar};
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status,
    THE_PARAM_TOLERANCE,
};
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use crate::extrema_pc_core::{NeighborStep, ValueFn};
use crate::extrema_pc2d::{self, EllipseFrame, canonical_roots, lift};
use crate::plane_projection as projlib;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis3, Direction3, Ellipse3, Plane3, Point3, Vector3};
use rustycad_tkmath::math_root::trigonometric;
use std::convert::Infallible;
use std::f64::consts::PI;

/// Point-ellipse extrema (`ExtremaPC_Ellipse`): solved by `ExtremaPC2d_Ellipse` in the
/// ellipse plane; when that fails, the trigonometric equation of the 2D evaluator is
/// solved for the point projected on the plane, and each extremum is kept once inside
/// the domain (no periodic repetition, unlike the 2D evaluator);
/// `Ellipse::new(ellipse).perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct Ellipse {
    ellipse: Ellipse3,
    domain: Option<Domain1D>,
    result: ExtremaResult,
}

impl Ellipse {
    /// Evaluator on the full ellipse (`ExtremaPC_Ellipse(theEllipse)`); `Ellipse::new(ellipse)`.
    pub fn new(ellipse: Ellipse3) -> Self {
        Self {
            ellipse,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC_Ellipse(theEllipse, theDomain)`);
    /// `Ellipse::with_domain(ellipse, Domain1D::new(0.0, PI))`.
    pub fn with_domain(ellipse: Ellipse3, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(ellipse)
        }
    }
    /// The point at `u` (`Value`, ElCLib::Value); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point3 {
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
    pub fn ellipse(&self) -> &Ellipse3 {
        &self.ellipse
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.clear();
            self.result.status = Status::InvalidInput;
            return Ok(&self.result);
        }
        if self.domain.is_some_and(|domain| domain.min == domain.max) {
            self.result.clear();
            self.result.status = Status::NoSolution;
            return Ok(&self.result);
        }
        if self.perform_in_plane(point, tol, mode)? {
            return Ok(&self.result);
        }
        let domain: Domain1D = self.domain.unwrap_or(Domain1D::new(0.0, 2.0 * PI));
        self.perform_core(point, &domain, tol, mode);
        Ok(&self.result)
    }
    /// The interior extrema and the extremal ends of an arc (`PerformWithEndpoints`);
    /// `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        analytic::perform_with_endpoints(self, point, tol, mode)
    }
    // ExtremaPC2d_Ellipse in the ellipse plane.
    fn perform_in_plane(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<bool, Failure> {
        let plane = Plane3::from_position(Axis3::from_axis2(self.ellipse.position()));
        let query = PlanarQuery {
            point2d: projlib::project_point(&plane, point),
            point3d: point,
            plane: &plane,
            tol,
            mode,
            with_endpoints: false,
        };
        let ellipse2d = projlib::project_ellipse(&plane, &self.ellipse)?;
        let ellipse = self.ellipse;
        let evaluator = ValueFn(|u| Ok::<Point3, Infallible>(elclib::value(u, &ellipse)));
        let Ok(done) = perform_planar::<extrema_pc2d::Ellipse, _>(
            ellipse2d,
            self.domain,
            &query,
            &evaluator,
            &mut self.result,
        );
        Ok(done)
    }
    // `performCore`: the trigonometric equation for the point projected on the plane.
    fn perform_core(&mut self, point: Point3, domain: &Domain1D, tol: f64, mode: SearchMode) {
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
        let roots = trigonometric([0.0, ko2, ko3, ko4, 0.0], 0.0, 2.0 * PI, 1.5e-12);
        if roots.infinite_roots {
            self.result.status = Status::InfiniteSolutions;
            let on_curve: Point3 = elclib::value(0.0, &self.ellipse);
            self.result.infinite_square_distance = point.square_distance(&on_curve);
            return;
        }
        if !roots.is_done() {
            self.result.status = Status::NumericalError;
            return;
        }
        let canonical: Vec<f64> = canonical_roots(&roots.roots[..roots.nb_roots]);
        self.classify(point, domain, frame, &canonical, mode);
    }
    // The local frame of the point projected on the plane; `None` after setting
    // InfiniteSolutions for a point on the axis of a circular ellipse.
    fn frame(&mut self, point: Point3) -> Option<EllipseFrame> {
        let center: Point3 = self.ellipse.location();
        let axis: Direction3 = self.ellipse.axis().direction();
        let height: f64 = Vector3::between(center, point).dot(Vector3::from(axis));
        let projected: Point3 = point.translated(Vector3::from(axis) * (-height));
        let (a, b) = (self.ellipse.major_radius(), self.ellipse.minor_radius());
        let center_to_projection: Vector3 = Vector3::between(center, projected);
        if center_to_projection.magnitude() <= gp::resolution() && a == b {
            // Point at center of a circle - infinite solutions
            self.result.status = Status::InfiniteSolutions;
            self.result.infinite_square_distance = a * a + height * height;
            return None;
        }
        let x: f64 = center_to_projection.dot(Vector3::from(self.ellipse.x_axis().direction()));
        let y: f64 = center_to_projection.dot(Vector3::from(self.ellipse.y_axis().direction()));
        Some(EllipseFrame { a, b, x, y })
    }
    // Classifies the canonical roots cyclically and adds the extrema.
    fn classify(
        &mut self,
        point: Point3,
        domain: &Domain1D,
        frame: EllipseFrame,
        roots: &[f64],
        mode: SearchMode,
    ) {
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
            if is_minimum || is_maximum {
                self.add_extremum(point, domain, (root, is_minimum), mode);
            }
        }
        self.result.status = super::found_status(&self.result);
    }
    // The addExtremum lambda: lift into the domain, skip outside or duplicate
    // parameters, filter by mode.
    fn add_extremum(
        &mut self,
        point: Point3,
        domain: &Domain1D,
        (root, is_min): (f64, bool),
        mode: SearchMode,
    ) {
        let u: f64 = lift(root, domain.min, THE_PARAM_TOLERANCE);
        if u < domain.min - THE_PARAM_TOLERANCE || u > domain.max + THE_PARAM_TOLERANCE {
            return;
        }
        let on_curve: Point3 = elclib::value(u, &self.ellipse);
        let duplicate: bool = self
            .result
            .extrema
            .iter()
            .any(|extremum| (extremum.parameter - u).abs() < THE_PARAM_TOLERANCE);
        if duplicate {
            return;
        }
        let square_distance: f64 = point.square_distance(&on_curve);
        if (mode == SearchMode::Min && !is_min) || (mode == SearchMode::Max && is_min) {
            return;
        }
        let extremum = ExtremumResult::new(u, on_curve, square_distance, is_min, !is_min);
        self.result.extrema.push(extremum);
    }
}

impl CurveEvaluator<Point3> for Ellipse {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point3, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point3> for Ellipse {
    const IS_PERIODIC: bool = true;
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
