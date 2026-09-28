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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Line.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_Line`: the projection of a 3D point on a line.
use super::planar::{PlanarQuery, perform_planar};
use super::{CurveEvaluator, Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status};
use crate::extrema_pc_core::analytic::{self, AnalyticEvaluator};
use crate::extrema_pc_core::{NeighborStep, ValueFn};
use crate::extrema_pc2d;
use crate::plane_projection as projlib;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Line3, Plane3, Point3, Vector3};
use std::convert::Infallible;

/// Point-line extrema (`ExtremaPC_Line`): solved by `ExtremaPC2d_Line` in the plane of
/// [`super::line_plane`], or by `u = (P - O) . D` when the 2D solve fails;
/// `Line::new(line).perform(point, 1e-9, SearchMode::MinMax)?.nb_ext() == 1`.
pub struct Line {
    line: Line3,
    domain: Option<Domain1D>,
    result: ExtremaResult,
}

/// `myLine.Location().Translated(theU * gp_Vec(myLine.Direction()))`.
fn line_value(line: &Line3, u: f64) -> Point3 {
    line.location()
        .translated(Vector3::from(line.direction()) * u)
}

impl Line {
    /// Evaluator on the unbounded line (`ExtremaPC_Line(theLine)`); `Line::new(line)`.
    pub fn new(line: Line3) -> Self {
        Self {
            line,
            domain: None,
            result: ExtremaResult::default(),
        }
    }
    /// Evaluator on `domain` (`ExtremaPC_Line(theLine, theDomain)`);
    /// `Line::with_domain(line, Domain1D::new(0.0, 1.0))`.
    pub fn with_domain(line: Line3, domain: Domain1D) -> Self {
        Self {
            domain: Some(domain),
            ..Self::new(line)
        }
    }
    /// The point at `u` (`Value`); `extrema.value(0.5)`.
    pub fn value(&self, u: f64) -> Point3 {
        line_value(&self.line, u)
    }
    /// True with a domain (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        self.domain.is_some()
    }
    /// The domain, `None` when unbounded (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> Option<&Domain1D> {
        self.domain.as_ref()
    }
    /// The line (`Line`); `extrema.line()`.
    pub fn line(&self) -> &Line3 {
        &self.line
    }
    /// The interior extremum (`Perform`); Max mode finds none;
    /// `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
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
        self.perform_direct(point, tol, mode);
        Ok(&self.result)
    }
    /// The interior extremum and the extremal ends of a bounded line
    /// (`PerformWithEndpoints`); `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        analytic::perform_with_endpoints(self, point, tol, mode)
    }
    // The 2D evaluator in the line's plane.
    fn perform_in_plane(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<bool, Failure> {
        let plane: Plane3 = super::line_plane(&self.line)?;
        let query = PlanarQuery {
            point2d: projlib::project_point(&plane, point),
            point3d: point,
            plane: &plane,
            tol,
            mode,
            with_endpoints: false,
        };
        let line2d = projlib::project_line(&plane, &self.line)?;
        let line = self.line;
        let evaluator = ValueFn(|u| Ok::<Point3, Infallible>(line_value(&line, u)));
        let Ok(done) = perform_planar::<extrema_pc2d::Line, _>(
            line2d,
            self.domain,
            &query,
            &evaluator,
            &mut self.result,
        );
        Ok(done)
    }
    // The projection `u = (P - O) . D` when the 2D solve fails.
    fn perform_direct(&mut self, point: Point3, tol: f64, mode: SearchMode) {
        if mode == SearchMode::Max {
            self.result.status = Status::NoSolution;
            return;
        }
        let direction: Vector3 = Vector3::from(self.line.direction());
        let origin: Point3 = self.line.location();
        let mut u: f64 = Vector3::between(origin, point).dot(direction);
        if let Some(domain) = self.domain {
            if u < domain.min - tol || u > domain.max + tol {
                // Projection is outside bounds - no interior extremum
                self.result.status = Status::NoSolution;
                return;
            }
            u = crate::extrema_pc2d::std_clamp(u, domain.min, domain.max);
        }
        let on_line: Point3 = origin.translated(direction * u);
        let extremum =
            ExtremumResult::new(u, on_line, point.square_distance(&on_line), true, false);
        self.result.extrema.push(extremum);
        self.result.status = Status::OK;
    }
}

impl CurveEvaluator<Point3> for Line {
    type Error = Infallible;
    fn evaluate(&self, u: f64) -> Result<Point3, Infallible> {
        Ok(self.value(u))
    }
}

impl AnalyticEvaluator<Point3> for Line {
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
