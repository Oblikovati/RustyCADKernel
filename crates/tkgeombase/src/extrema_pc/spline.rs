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
// Rust translation of the shared body of ModelingData/TKGeomBase/ExtremaPC/
// ExtremaPC_BezierCurve.cxx and ExtremaPC_BSplineCurve.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The body the 3D Bezier and BSpline evaluators share: the curve, its adaptor, the
//! domain and a partition built once at construction (so, unlike ExtremaPC2d, later
//! pole edits reach the evaluation but not the partition), the planar 2D solve, and
//! the endpoint flow. [`SplineKind`] holds what differs.
use super::{Domain1D, ExtremaResult, GridEvaluator, SearchMode, Status, add_endpoint_extrema};
use crate::extrema_pc_core::{CurveEvaluator, is_valid_tolerance};
use crate::extrema_pc2d;
use crate::plane_projection as projlib;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkmath::math::Vector;

/// The kind-specific parts of a 3D spline evaluator.
pub(crate) trait SplineKind {
    /// The Geom curve class.
    type Geom: GeomCurve + RuntimeTyped;
    /// The ExtremaPC2d evaluator of the projected curve.
    type Planar;
    /// `ProjectBezier` or `ProjectBSpline`, then the 2D evaluator over `domain`.
    fn project(
        curve: &Handle<Self::Geom>,
        plane: &mut Plane3,
        domain: Domain1D,
    ) -> Result<Option<Self::Planar>, Failure>;
    /// `Perform` or `PerformWithEndpoints` of the 2D evaluator.
    fn run_planar(
        evaluator: &mut Self::Planar,
        point: rustycad_tkmath::gp::Point2,
        tol: f64,
        mode: SearchMode,
        with_endpoints: bool,
    ) -> Result<&extrema_pc2d::ExtremaResult, Failure>;
    /// `buildParams`'s partition of a valid finite domain.
    fn partition(curve: &Self::Geom, domain: &Domain1D) -> Result<Vector<'static>, Failure>;
}

/// The curve, evaluated by AddEndpointExtrema apart from the evaluator's result.
pub(crate) struct SplineShape<G: RuntimeTyped>(pub Handle<G>);

impl<G: GeomCurve + RuntimeTyped> CurveEvaluator<Point3> for SplineShape<G> {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point3, Failure> {
        let curve = self.0.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "ExtremaPC: Value on a null curve; expected a curve",
                "",
            )
        })?;
        curve.value(u)
    }
}

/// The shared state of `ExtremaPC_BezierCurve` and `ExtremaPC_BSplineCurve`.
pub(crate) struct SplineExtrema<K: SplineKind> {
    pub shape: SplineShape<K::Geom>,
    adaptor: GeomAdaptorCurve,
    pub domain: Domain1D,
    evaluator: GridEvaluator,
}

impl<K: SplineKind> SplineExtrema<K> {
    /// The constructors: load the adaptor and build the partition of a non-null curve.
    pub fn new(curve: Handle<K::Geom>, domain: Domain1D) -> Result<Self, Failure> {
        let mut result = Self {
            shape: SplineShape(curve),
            adaptor: GeomAdaptorCurve::default(),
            domain,
            evaluator: GridEvaluator::default(),
        };
        if let Some(geometry) = result.shape.0.get() {
            result.adaptor.load(&result.shape.0.cast())?;
            if result.domain.is_valid() && result.domain.is_finite(1.0e100) {
                let params: Vector<'static> = K::partition(geometry, &result.domain)?;
                result.evaluator.set_params(&params);
            }
        }
        Ok(result)
    }
    /// `Perform`: the planar 2D solve, else the grid evaluator on the adaptor.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        if self.shape.0.is_null() {
            return Ok(not_done(self.evaluator.result()));
        }
        if is_valid_tolerance(tol)
            && self.domain.is_valid()
            && self.perform_planar(point, tol, mode, false)?
        {
            return Ok(self.evaluator.result());
        }
        self.evaluator
            .perform(&self.adaptor, &point, &self.domain, tol, mode)
    }
    /// `PerformWithEndpoints`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        if self.shape.0.is_null() {
            return Ok(not_done(self.evaluator.result()));
        }
        if is_valid_tolerance(tol)
            && self.domain.is_valid()
            && self.perform_planar(point, tol, mode, true)?
        {
            return Ok(self.evaluator.result());
        }
        self.perform(point, tol, mode)?;
        let status: Status = self.evaluator.result().status;
        if status == Status::OK || status == Status::NoSolution {
            let is_closed: bool = GridEvaluator::is_closed_domain(&self.adaptor, &self.domain);
            let result: &mut ExtremaResult = self.evaluator.result();
            add_endpoint_extrema(result, &point, &self.domain, &self.shape, mode, is_closed)?;
            if !result.extrema.is_empty() {
                result.status = Status::OK;
            }
        }
        Ok(self.evaluator.result())
    }
    // `performPlanar`: the 2D evaluator of the exact projection, converted to 3D.
    fn perform_planar(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
        ends: bool,
    ) -> Result<bool, Failure> {
        let mut plane = Plane3::default();
        let domain = Domain1D::new(self.domain.min, self.domain.max);
        let Some(mut planar) = K::project(&self.shape.0, &mut plane, domain)? else {
            return Ok(false);
        };
        let point2d = projlib::project_point(&plane, point);
        let mode2d: SearchMode = super::to_search_mode_2d(mode);
        let result2d = K::run_planar(&mut planar, point2d, tol, mode2d, ends)?;
        super::convert_planar_result(
            result2d,
            &point,
            &plane,
            &self.shape,
            self.evaluator.result(),
        )
    }
}

/// `Result().Clear(); Status = NotDone` of a null curve.
pub(crate) fn not_done(result: &mut ExtremaResult) -> &ExtremaResult {
    result.clear();
    result.status = Status::NotDone;
    result
}
