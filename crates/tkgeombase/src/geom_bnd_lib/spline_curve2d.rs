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
// Rust translation of GeomBndLib_BezierCurve2d and GeomBndLib_BSplineCurve2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_BezierCurve2d` and `GeomBndLib_BSplineCurve2d`: unlike the 3D classes,
//! the 2D `Box` is the hull of the poles of the curve segmented to the range.
use super::live;
use super::spline::curve_box_optimal;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::bvh::{std_max, std_min};
use rustycad_tkmath::gp::Point2;

/// A `Geom2dAdaptor_Curve` on a typed curve handle; `adaptor2d_of(&bezier)?`.
pub(super) fn adaptor2d_of<T: Geom2dCurve + RuntimeTyped>(
    curve: &Handle<T>,
) -> Result<Geom2dAdaptorCurve, Failure> {
    Geom2dAdaptorCurve::with_curve(&curve.cast::<dyn Geom2dCurve>())
}

// The box of `poles(1..=nb_poles)` enlarged by `tolerance`.
fn pole_hull(
    poles: impl Iterator<Item = Result<Point2, Failure>>,
    tolerance: f64,
) -> Result<BoundingBox2, Failure> {
    let mut bounds: BoundingBox2 = BoundingBox2::default();
    for pole in poles {
        bounds.add_point(pole?);
    }
    bounds.enlarge(tolerance);
    Ok(bounds)
}

// Whether the trimmed range moves off the curve bounds by more than PConfusion.
fn is_trimmed((first, last): (f64, f64), (trim1, trim2): (f64, f64)) -> bool {
    (first - trim1).abs() > precision::p_confusion()
        || (last - trim2).abs() > precision::p_confusion()
}

/// `GeomBndLib_BezierCurve2d`: the box of a `Geom2d_BezierCurve`.
pub struct BezierCurve2d {
    geom: Handle<Geom2dBezierCurve>,
}

impl BezierCurve2d {
    /// The evaluator of `curve`; `BezierCurve2d::new(handle)`.
    pub fn new(curve: Handle<Geom2dBezierCurve>) -> Self {
        Self { geom: curve }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<Geom2dBezierCurve> {
        &self.geom
    }
    /// `Box(Tol)`: the whole curve; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let curve = live(&self.geom, "GeomBndLib_BezierCurve2d")?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `Box(U1, U2, Tol)`: the range clamped to the curve; a copy segmented to it when
    /// it differs by more than PConfusion; the hull of the poles, enlarged by `tol`;
    /// `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        let curve = live(&self.geom, "GeomBndLib_BezierCurve2d")?;
        let bounds: (f64, f64) = (curve.first_parameter(), curve.last_parameter());
        let trim: (f64, f64) = (std_max(u1, bounds.0), std_min(u2, bounds.1));
        if !is_trimmed(bounds, trim) {
            return pole_hull((1..=curve.nb_poles()).map(|i| curve.pole(i)), tolerance);
        }
        let copy: Geom2dBezierCurve = Geom2dBezierCurve::copy_of(curve)?;
        copy.segment(trim.0, trim.1)?;
        pole_hull((1..=copy.nb_poles()).map(|i| copy.pole(i)), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: `CurveBoxOptimal` on a `Geom2dAdaptor_Curve`;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        let adaptor: Geom2dAdaptorCurve = adaptor2d_of(&self.geom)?;
        let adaptor: &dyn Adaptor2dCurve2d = &adaptor;
        curve_box_optimal(adaptor, u1, u2, tolerance)
    }
}

/// `GeomBndLib_BSplineCurve2d`: the box of a `Geom2d_BSplineCurve`.
pub struct BSplineCurve2d {
    geom: Handle<Geom2dBSplineCurve>,
}

impl BSplineCurve2d {
    /// The evaluator of `curve`; `BSplineCurve2d::new(handle)`.
    pub fn new(curve: Handle<Geom2dBSplineCurve>) -> Self {
        Self { geom: curve }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<Geom2dBSplineCurve> {
        &self.geom
    }
    /// `Box(Tol)`: the whole curve; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let curve = live(&self.geom, "GeomBndLib_BSplineCurve2d")?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `Box(U1, U2, Tol)`: the range clamped to the curve (the whole curve when the
    /// clamped range is reversed); a copy segmented to it (default PConfusion tolerance)
    /// when it differs; the hull of the poles; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        let curve = live(&self.geom, "GeomBndLib_BSplineCurve2d")?;
        let bounds: (f64, f64) = (curve.first_parameter(), curve.last_parameter());
        let mut trim: (f64, f64) = (std_max(u1, bounds.0), std_min(u2, bounds.1));
        if trim.1 < trim.0 {
            trim = bounds;
        }
        if !is_trimmed(bounds, trim) {
            return pole_hull((1..=curve.nb_poles()).map(|i| curve.pole(i)), tolerance);
        }
        let copy: Geom2dBSplineCurve = Geom2dBSplineCurve::copy_of(curve)?;
        copy.segment(trim.0, trim.1, precision::p_confusion())?;
        pole_hull((1..=copy.nb_poles()).map(|i| copy.pole(i)), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: `CurveBoxOptimal` on a `Geom2dAdaptor_Curve`;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        let adaptor: Geom2dAdaptorCurve = adaptor2d_of(&self.geom)?;
        let adaptor: &dyn Adaptor2dCurve2d = &adaptor;
        curve_box_optimal(adaptor, u1, u2, tolerance)
    }
}
