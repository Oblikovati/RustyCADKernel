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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Circle.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_Circle`: the nearest and farthest points of a circle in space.
use super::planar::{PlanarQuery, perform_planar};
use super::{CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status};
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use crate::extrema_pc_core::{NeighborStep, ValueFn};
use crate::extrema_pc2d;
use crate::plane_projection as projlib;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis3, Circle3, Direction3, Plane3, Point3, Vector3};
use std::convert::Infallible;
use std::f64::consts::PI;

/// Point-circle extrema (`ExtremaPC_Circle`): solved by `ExtremaPC2d_Circle` in the
/// circle plane; when that fails, the angle of the projected point is the minimum and
/// its opposite the maximum, kept within the domain widened by `tol / R`. A point on the
/// axis (or a zero radius) has infinitely many solutions;
/// `Circle::new(circle).perform(point, 1e-9, SearchMode::MinMax)?.nb_ext() == 2`.
pub struct Circle {
    circle: Circle3,
    domain: Option<Domain1D>,
    result: ExtremaResult,
}

impl Circle {
    /// Evaluator on the full circle (`ExtremaPC_Circle(theCircle)`); `Circle::new(circle)`.
    pub fn new(circle: Circle3) -> Self {
        Self {
            circle,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC_Circle(theCircle, theDomain)`);
    /// `Circle::with_domain(circle, Domain1D::new(0.0, PI))`.
    pub fn with_domain(circle: Circle3, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(circle)
        }
    }
    /// The point at `u` (`Value`, ElCLib::Value); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point3 {
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
    pub fn circle(&self) -> &Circle3 {
        &self.circle
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.result.clear();
        if super::is_invalid_input(tol, &point, self.domain.as_ref()) {
            self.result.status = Status::InvalidInput;
            return Ok(&self.result);
        }
        if self.perform_in_plane(point, tol, mode)? {
            return Ok(&self.result);
        }
        self.perform_direct(point, tol, mode)?;
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
    // ExtremaPC2d_Circle in the circle plane.
    fn perform_in_plane(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<bool, Failure> {
        let plane = Plane3::from_position(Axis3::from_axis2(self.circle.position()));
        let query = PlanarQuery {
            point2d: projlib::project_point(&plane, point),
            point3d: point,
            plane: &plane,
            tol,
            mode,
            with_endpoints: false,
        };
        let circle2d = projlib::project_circle(&plane, &self.circle)?;
        let circle = self.circle;
        let evaluator = ValueFn(|u| Ok::<Point3, Infallible>(elclib::value(u, &circle)));
        let Ok(done) = perform_planar::<extrema_pc2d::Circle, _>(
            circle2d,
            self.domain,
            &query,
            &evaluator,
            &mut self.result,
        );
        Ok(done)
    }
    // The direct 3D algorithm, when the 2D solve fails.
    fn perform_direct(&mut self, point: Point3, tol: f64, mode: SearchMode) -> Result<(), Failure> {
        if self.circle.radius() <= gp::resolution() {
            self.result.status = Status::InfiniteSolutions;
            self.result.infinite_square_distance = point.square_distance(&self.circle.location());
            return Ok(());
        }
        // Step 1: Project point P onto the circle plane
        let center: Point3 = self.circle.location();
        let axis: Direction3 = self.circle.axis().direction();
        let height: f64 = Vector3::between(center, point).dot(Vector3::from(axis));
        let projected: Point3 = point.translated(Vector3::from(axis) * (-height));
        // Step 2: Check for degenerate case - point projects to center
        let center_to_projection: Vector3 = Vector3::between(center, projected);
        if center_to_projection.magnitude() <= gp::resolution() {
            // Point is on the circle axis - all points on circle are equidistant
            self.result.status = Status::InfiniteSolutions;
            let radius: f64 = self.circle.radius();
            self.result.infinite_square_distance = radius * radius + height * height;
            return Ok(());
        }
        let solutions: [f64; 2] = self.solutions(center_to_projection, axis)?;
        self.add_solutions(point, solutions, tol, mode);
        Ok(())
    }
    // The minimum angle (snapped to +-PI), its opposite, both lifted into the domain.
    fn solutions(
        &self,
        center_to_projection: Vector3,
        axis: Direction3,
    ) -> Result<[f64; 2], Failure> {
        let direction: Direction3 = Direction3::from_vector(center_to_projection)?;
        let mut us1: f64 = self
            .circle
            .x_axis()
            .direction()
            .angle_with_ref(direction, axis);
        let ang_tol: f64 = precision::angular();
        if us1 + PI < ang_tol {
            us1 = -PI;
        } else if us1 - PI > -ang_tol {
            us1 = PI;
        }
        let mut us2: f64 = us1 + PI;
        if let Some(domain) = self.domain {
            let lift_tolerance: f64 = precision::angular();
            us1 = crate::extrema_pc2d::lift(us1, domain.min, lift_tolerance);
            us2 = crate::extrema_pc2d::lift(us2, domain.min, lift_tolerance);
        }
        Ok([us1, us2])
    }
    // The minimum (index 0) and the maximum (index 1) in the search mode, within the
    // domain widened by the angular tolerance `tol / R`.
    fn add_solutions(&mut self, point: Point3, solutions: [f64; 2], tol: f64, mode: SearchMode) {
        let mut tol_u: f64 = precision::angular();
        if self.domain.is_some() && self.circle.radius() > gp::resolution() {
            tol_u = tol / self.circle.radius();
        }
        let start: usize = if mode == SearchMode::Max { 1 } else { 0 };
        let end: usize = if mode == SearchMode::Min { 1 } else { 2 };
        for (index, &u) in solutions.iter().enumerate().take(end).skip(start) {
            let inside: bool = self
                .domain
                .is_none_or(|domain| u >= domain.min - tol_u && u <= domain.max + tol_u);
            if !inside {
                continue;
            }
            let on_curve: Point3 = elclib::value(u, &self.circle);
            let square_distance: f64 = point.square_distance(&on_curve);
            self.result.extrema.push(ExtremumResult::new(
                u,
                on_curve,
                square_distance,
                index == 0,
                index == 1,
            ));
        }
        self.result.status = super::found_status(&self.result);
    }
}

impl CurveEvaluator<Point3> for Circle {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point3, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point3> for Circle {
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
