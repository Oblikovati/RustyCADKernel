// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.cxx (the per-order branches of EvalD0..EvalD3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The four native evaluators repeat one switch with order-specific calls; each
//! order here supplies those calls, so the switch is written once.
use super::curve_state::CurveData;
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::errors::{undefined_derivative, undefined_value};
use crate::geom::offset_curve_utils::{evaluate_d0, evaluate_d1, evaluate_d2, evaluate_d3};
use crate::geom::{GeomBSplineCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// The order-specific calls of one `EvalDk` of `GeomAdaptor_Curve`.
pub(super) trait EvalOrder {
    /// `gp_Pnt` or `Geom_Curve::ResDk`.
    type Output;
    /// ElCLib on elementary data, `None` for the other kinds.
    fn elementary(data: &CurveData, u: f64) -> Option<Self::Output>;
    /// `myCurve->EvalDk(U)`.
    fn curve(curve: &dyn GeomCurve, u: f64) -> Result<Self::Output, Failure>;
    /// `LocalDk` on the boundary span.
    fn local(
        curve: &GeomBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<Self::Output, Failure>;
    /// `Cache->Dk`.
    fn cached(cache: &Cache, u: f64) -> Result<Self::Output, Failure>;
    /// `Geom_OffsetCurveUtils::EvaluateDk`; `None` when the offset is undefined.
    fn offset(
        u: f64,
        basis: &dyn Adaptor3dCurve,
        direction: Direction3,
        offset: f64,
    ) -> Result<Option<Self::Output>, Failure>;
    /// The exception of a failed offset evaluation.
    fn failure() -> Failure;
}

const ZERO: Vector3 = Vector3::new(0.0, 0.0, 0.0);

/// `EvalD0`.
pub(super) struct EvalD0;
impl EvalOrder for EvalD0 {
    type Output = Point3;
    fn elementary(data: &CurveData, u: f64) -> Option<Point3> {
        Some(match data {
            CurveData::Line(line) => elclib::value(u, line),
            CurveData::Circle(circle) => elclib::value(u, circle),
            CurveData::Ellipse(ellipse) => elclib::value(u, ellipse),
            CurveData::Hyperbola(hyperbola) => elclib::value(u, hyperbola),
            CurveData::Parabola(parabola) => elclib::value(u, parabola),
            _ => return None,
        })
    }
    fn curve(curve: &dyn GeomCurve, u: f64) -> Result<Point3, Failure> {
        curve.eval_d0(u)
    }
    fn local(curve: &GeomBSplineCurve, u: f64, start: i32, finish: i32) -> Result<Point3, Failure> {
        curve.local_d0(u, start, finish)
    }
    fn cached(cache: &Cache, u: f64) -> Result<Point3, Failure> {
        cache.d0::<Point3>(u)
    }
    fn offset(
        u: f64,
        basis: &dyn Adaptor3dCurve,
        direction: Direction3,
        offset: f64,
    ) -> Result<Option<Point3>, Failure> {
        evaluate_d0(u, basis, direction, offset)
    }
    fn failure() -> Failure {
        undefined_value("GeomAdaptor_Curve::EvalD0: evaluation failed")
    }
}

/// `EvalD1`.
pub(super) struct EvalD1;
impl EvalOrder for EvalD1 {
    type Output = GeomCurveD1;
    fn elementary(data: &CurveData, u: f64) -> Option<GeomCurveD1> {
        let (point, d1) = match data {
            CurveData::Line(line) => elclib::d1(u, line),
            CurveData::Circle(circle) => elclib::d1(u, circle),
            CurveData::Ellipse(ellipse) => elclib::d1(u, ellipse),
            CurveData::Hyperbola(hyperbola) => elclib::d1(u, hyperbola),
            CurveData::Parabola(parabola) => elclib::d1(u, parabola),
            _ => return None,
        };
        Some(GeomCurveD1 { point, d1 })
    }
    fn curve(curve: &dyn GeomCurve, u: f64) -> Result<GeomCurveD1, Failure> {
        curve.eval_d1(u)
    }
    fn local(
        curve: &GeomBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<GeomCurveD1, Failure> {
        let (point, d1) = curve.local_d1(u, start, finish)?;
        Ok(GeomCurveD1 { point, d1 })
    }
    fn cached(cache: &Cache, u: f64) -> Result<GeomCurveD1, Failure> {
        let (point, d1) = cache.d1::<Point3>(u)?;
        Ok(GeomCurveD1 { point, d1 })
    }
    fn offset(
        u: f64,
        basis: &dyn Adaptor3dCurve,
        direction: Direction3,
        offset: f64,
    ) -> Result<Option<GeomCurveD1>, Failure> {
        evaluate_d1(u, basis, direction, offset)
    }
    fn failure() -> Failure {
        undefined_derivative("GeomAdaptor_Curve::EvalD1: evaluation failed")
    }
}

/// `EvalD2`: a line has no ElCLib D2, its second derivative is set to zero.
pub(super) struct EvalD2;
impl EvalOrder for EvalD2 {
    type Output = GeomCurveD2;
    fn elementary(data: &CurveData, u: f64) -> Option<GeomCurveD2> {
        let (point, d1, d2) = match data {
            CurveData::Line(line) => {
                let (point, d1) = elclib::d1(u, line);
                (point, d1, ZERO)
            }
            CurveData::Circle(circle) => elclib::d2(u, circle),
            CurveData::Ellipse(ellipse) => elclib::d2(u, ellipse),
            CurveData::Hyperbola(hyperbola) => elclib::d2(u, hyperbola),
            CurveData::Parabola(parabola) => elclib::d2(u, parabola),
            _ => return None,
        };
        Some(GeomCurveD2 { point, d1, d2 })
    }
    fn curve(curve: &dyn GeomCurve, u: f64) -> Result<GeomCurveD2, Failure> {
        curve.eval_d2(u)
    }
    fn local(
        curve: &GeomBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<GeomCurveD2, Failure> {
        let (point, d1, d2) = curve.local_d2(u, start, finish)?;
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn cached(cache: &Cache, u: f64) -> Result<GeomCurveD2, Failure> {
        let (point, d1, d2) = cache.d2::<Point3>(u)?;
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn offset(
        u: f64,
        basis: &dyn Adaptor3dCurve,
        direction: Direction3,
        offset: f64,
    ) -> Result<Option<GeomCurveD2>, Failure> {
        evaluate_d2(u, basis, direction, offset)
    }
    fn failure() -> Failure {
        undefined_derivative("GeomAdaptor_Curve::EvalD2: evaluation failed")
    }
}

/// `EvalD3`: a line uses D1 and a parabola D2, the missing derivatives set to zero.
pub(super) struct EvalD3;
impl EvalOrder for EvalD3 {
    type Output = GeomCurveD3;
    fn elementary(data: &CurveData, u: f64) -> Option<GeomCurveD3> {
        let (point, d1, d2, d3) = match data {
            CurveData::Line(line) => {
                let (point, d1) = elclib::d1(u, line);
                (point, d1, ZERO, ZERO)
            }
            CurveData::Circle(circle) => elclib::d3(u, circle),
            CurveData::Ellipse(ellipse) => elclib::d3(u, ellipse),
            CurveData::Hyperbola(hyperbola) => elclib::d3(u, hyperbola),
            CurveData::Parabola(parabola) => {
                let (point, d1, d2) = elclib::d2(u, parabola);
                (point, d1, d2, ZERO)
            }
            _ => return None,
        };
        Some(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn curve(curve: &dyn GeomCurve, u: f64) -> Result<GeomCurveD3, Failure> {
        curve.eval_d3(u)
    }
    fn local(
        curve: &GeomBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<GeomCurveD3, Failure> {
        let (point, d1, d2, d3) = curve.local_d3(u, start, finish)?;
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn cached(cache: &Cache, u: f64) -> Result<GeomCurveD3, Failure> {
        let (point, d1, d2, d3) = cache.d3::<Point3>(u)?;
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn offset(
        u: f64,
        basis: &dyn Adaptor3dCurve,
        direction: Direction3,
        offset: f64,
    ) -> Result<Option<GeomCurveD3>, Failure> {
        evaluate_d3(u, basis, direction, offset)
    }
    fn failure() -> Failure {
        undefined_derivative("GeomAdaptor_Curve::EvalD3: evaluation failed")
    }
}
