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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_BezierCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_BezierCurve`: numerical extrema on a 3D Bezier curve.
use super::spline::{SplineExtrema, SplineKind};
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, GridEvaluator, SearchMode,
    THE_BEZIER_DEGREE_MULTIPLIER, THE_BEZIER_MIN_SAMPLES, project_bezier,
};
use crate::extrema_pc2d;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBezierCurve;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::gp::{Plane3, Point2, Point3};
use rustycad_tkmath::math::Vector;

/// The Bezier parts of the shared spline evaluator.
pub(crate) struct BezierKind;

impl SplineKind for BezierKind {
    type Geom = GeomBezierCurve;
    type Planar = extrema_pc2d::BezierCurve;
    fn project(
        curve: &Handle<GeomBezierCurve>,
        plane: &mut Plane3,
        domain: Domain1D,
    ) -> Result<Option<extrema_pc2d::BezierCurve>, Failure> {
        let mut curve2d: Handle<Geom2dBezierCurve> = Handle::default();
        if !project_bezier(curve, plane, &mut curve2d)? {
            return Ok(None);
        }
        Ok(Some(extrema_pc2d::BezierCurve::with_domain(
            curve2d, domain,
        )))
    }
    fn run_planar(
        evaluator: &mut extrema_pc2d::BezierCurve,
        point: Point2,
        tol: f64,
        mode: SearchMode,
        with_endpoints: bool,
    ) -> Result<&extrema_pc2d::ExtremaResult, Failure> {
        if with_endpoints {
            return evaluator.perform_with_endpoints(point, tol, mode);
        }
        evaluator.perform(point, tol, mode)
    }
    fn partition(curve: &GeomBezierCurve, domain: &Domain1D) -> Result<Vector<'static>, Failure> {
        let nb_samples: usize = sample_count(curve.degree());
        GridEvaluator::build_uniform_params(domain.min, domain.max, nb_samples)
    }
}

// `max(THE_BEZIER_MIN_SAMPLES, THE_BEZIER_DEGREE_MULTIPLIER * (degree + 1))`.
fn sample_count(degree: i32) -> usize {
    THE_BEZIER_MIN_SAMPLES.max(THE_BEZIER_DEGREE_MULTIPLIER * (degree + 1) as usize)
}

/// Point-Bezier extrema in space (`ExtremaPC_BezierCurve`).
///
/// A Bezier curve with an exactly planar control polygon is solved by
/// `ExtremaPC2d_BezierCurve` in its plane; otherwise by the grid evaluator over
/// max(24, 3 (degree + 1)) uniform samples built at construction. A null curve gives
/// NotDone; `BezierCurve::new(curve)?.perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct BezierCurve(SplineExtrema<BezierKind>);

impl BezierCurve {
    /// Evaluator over the curve bounds (`ExtremaPC_BezierCurve(theCurve)`); a null curve
    /// keeps the domain `[0, 1]`; `BezierCurve::new(curve)?`.
    pub fn new(curve: Handle<GeomBezierCurve>) -> Result<Self, Failure> {
        let domain: Domain1D = match curve.get() {
            Some(bezier) => Domain1D::new(bezier.first_parameter(), bezier.last_parameter()),
            None => Domain1D::new(0.0, 1.0),
        };
        Self::with_domain(curve, domain)
    }
    /// Evaluator over `domain` (`ExtremaPC_BezierCurve(theCurve, theDomain)`);
    /// `BezierCurve::with_domain(curve, Domain1D::new(0.0, 0.5))?`.
    pub fn with_domain(curve: Handle<GeomBezierCurve>, domain: Domain1D) -> Result<Self, Failure> {
        SplineExtrema::new(curve, domain).map(Self)
    }
    /// The curve point at `u` (`Value`); `extrema.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<Point3, Failure> {
        self.0.shape.evaluate(u)
    }
    /// Always true: a Bezier curve is bounded (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        true
    }
    /// The domain (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> &Domain1D {
        &self.0.domain
    }
    /// The curve (`Curve`); `extrema.curve()`.
    pub fn curve(&self) -> &Handle<GeomBezierCurve> {
        &self.0.shape.0
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.0.perform(point, tol, mode)
    }
    /// The interior extrema and the extremal ends (`PerformWithEndpoints`);
    /// `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.0.perform_with_endpoints(point, tol, mode)
    }
}

impl CurveEvaluator<Point3> for BezierCurve {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point3, Failure> {
        self.value(u)
    }
}
