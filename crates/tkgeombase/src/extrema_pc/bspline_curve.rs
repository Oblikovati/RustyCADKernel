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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_BSplineCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_BSplineCurve`: numerical extrema on a 3D BSpline curve.
use super::spline::{SplineExtrema, SplineKind};
use super::{CurveEvaluator, Domain1D, ExtremaResult, SearchMode, project_bspline};
use crate::extrema_pc_core::knot_params::{KnotGrid, knot_aware_params};
use crate::extrema_pc2d;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::gp::{Plane3, Point2, Point3};
use rustycad_tkmath::math::Vector;

/// The BSpline parts of the shared spline evaluator.
pub(crate) struct BSplineKind;

impl SplineKind for BSplineKind {
    type Geom = GeomBSplineCurve;
    type Planar = extrema_pc2d::BSplineCurve;
    fn project(
        curve: &Handle<GeomBSplineCurve>,
        plane: &mut Plane3,
        domain: Domain1D,
    ) -> Result<Option<extrema_pc2d::BSplineCurve>, Failure> {
        let mut curve2d: Handle<Geom2dBSplineCurve> = Handle::default();
        if !project_bspline(curve, plane, &mut curve2d)? {
            return Ok(None);
        }
        Ok(Some(extrema_pc2d::BSplineCurve::with_domain(
            curve2d, domain,
        )))
    }
    fn run_planar(
        evaluator: &mut extrema_pc2d::BSplineCurve,
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
    fn partition(curve: &GeomBSplineCurve, domain: &Domain1D) -> Result<Vector<'static>, Failure> {
        let grid = KnotGrid {
            knots: curve.knots().iter().copied().collect(),
            degree: curve.degree(),
            period: if curve.is_periodic() {
                Some(curve.period()?)
            } else {
                None
            },
            class_name: "ExtremaPC_BSplineCurve",
        };
        knot_aware_params(&grid, domain)
    }
}

/// Point-BSpline extrema in space (`ExtremaPC_BSplineCurve`).
///
/// A BSpline curve with an exactly planar control polygon is solved by
/// `ExtremaPC2d_BSplineCurve` in its plane; otherwise by the grid evaluator over the
/// knot-aware partition built at construction (Standard_RangeError for a grid too large
/// for an `int` count). A null curve gives NotDone;
/// `BSplineCurve::new(curve)?.perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct BSplineCurve(SplineExtrema<BSplineKind>);

impl BSplineCurve {
    /// Evaluator over the curve bounds (`ExtremaPC_BSplineCurve(theCurve)`); a null curve
    /// keeps the domain `[0, 1]`; `BSplineCurve::new(curve)?`.
    pub fn new(curve: Handle<GeomBSplineCurve>) -> Result<Self, Failure> {
        let domain: Domain1D = match curve.get() {
            Some(bspline) => Domain1D::new(bspline.first_parameter(), bspline.last_parameter()),
            None => Domain1D::new(0.0, 1.0),
        };
        Self::with_domain(curve, domain)
    }
    /// Evaluator over `domain` (`ExtremaPC_BSplineCurve(theCurve, theDomain)`);
    /// `BSplineCurve::with_domain(curve, Domain1D::new(0.0, 0.5))?`.
    pub fn with_domain(curve: Handle<GeomBSplineCurve>, domain: Domain1D) -> Result<Self, Failure> {
        SplineExtrema::new(curve, domain).map(Self)
    }
    /// The curve point at `u` (`Value`); `extrema.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<Point3, Failure> {
        self.0.shape.evaluate(u)
    }
    /// Always true: a BSpline curve is bounded (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        true
    }
    /// The domain (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> &Domain1D {
        &self.0.domain
    }
    /// The curve (`Curve`); `extrema.curve()`.
    pub fn curve(&self) -> &Handle<GeomBSplineCurve> {
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

impl CurveEvaluator<Point3> for BSplineCurve {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point3, Failure> {
        self.value(u)
    }
}
