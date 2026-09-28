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
// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.cxx (the per-order branches of EvalD0..EvalD3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The four native evaluators repeat one switch with order-specific calls; each
//! order here supplies those calls, so the switch is written once.
use super::curve::Geom2dAdaptorCurve;
use super::curve_state::CurveData;
use crate::geom2d::errors::{undefined_derivative, undefined_value};
use crate::geom2d::offset_curve_utils::{evaluate_d0, evaluate_d1, evaluate_d2, evaluate_d3};
use crate::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The order-specific calls of one `EvalDk` of `Geom2dAdaptor_Curve`.
pub(super) trait EvalOrder {
    /// `gp_Pnt2d` or `Geom2d_Curve::ResDk`.
    type Output;
    /// ElCLib on elementary data, `None` for the other kinds.
    fn elementary(data: &CurveData, u: f64) -> Option<Self::Output>;
    /// `myCurve->EvalDk(U)`.
    fn curve(curve: &dyn Geom2dCurve, u: f64) -> Result<Self::Output, Failure>;
    /// `LocalDk` on the boundary span.
    fn local(
        curve: &Geom2dBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<Self::Output, Failure>;
    /// `Cache->Dk`.
    fn cached(cache: &Cache, u: f64) -> Result<Self::Output, Failure>;
    /// `Geom2d_OffsetCurveUtils::EvaluateDk`; `None` when the offset is undefined.
    fn offset(
        u: f64,
        basis: &Geom2dAdaptorCurve,
        offset: f64,
    ) -> Result<Option<Self::Output>, Failure>;
    /// The exception of a failed offset evaluation.
    fn failure() -> Failure;
}

const ZERO: Vector2 = Vector2::new(0.0, 0.0);

/// `EvalD0`.
pub(super) struct EvalD0;
impl EvalOrder for EvalD0 {
    type Output = Point2;
    fn elementary(data: &CurveData, u: f64) -> Option<Point2> {
        Some(match data {
            CurveData::Line(line) => elclib::value(u, line),
            CurveData::Circle(circle) => elclib::value(u, circle),
            CurveData::Ellipse(ellipse) => elclib::value(u, ellipse),
            CurveData::Hyperbola(hyperbola) => elclib::value(u, hyperbola),
            CurveData::Parabola(parabola) => elclib::value(u, parabola),
            _ => return None,
        })
    }
    fn curve(curve: &dyn Geom2dCurve, u: f64) -> Result<Point2, Failure> {
        curve.eval_d0(u)
    }
    fn local(
        curve: &Geom2dBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<Point2, Failure> {
        curve.local_d0(u, start, finish)
    }
    fn cached(cache: &Cache, u: f64) -> Result<Point2, Failure> {
        cache.d0::<Point2>(u)
    }
    fn offset(u: f64, basis: &Geom2dAdaptorCurve, offset: f64) -> Result<Option<Point2>, Failure> {
        evaluate_d0(u, basis, offset)
    }
    fn failure() -> Failure {
        undefined_value("Geom2dAdaptor_Curve::EvalD0: evaluation failed")
    }
}

/// `EvalD1`.
pub(super) struct EvalD1;
impl EvalOrder for EvalD1 {
    type Output = Geom2dCurveD1;
    fn elementary(data: &CurveData, u: f64) -> Option<Geom2dCurveD1> {
        let (point, d1) = match data {
            CurveData::Line(line) => elclib::d1(u, line),
            CurveData::Circle(circle) => elclib::d1(u, circle),
            CurveData::Ellipse(ellipse) => elclib::d1(u, ellipse),
            CurveData::Hyperbola(hyperbola) => elclib::d1(u, hyperbola),
            CurveData::Parabola(parabola) => elclib::d1(u, parabola),
            _ => return None,
        };
        Some(Geom2dCurveD1 { point, d1 })
    }
    fn curve(curve: &dyn Geom2dCurve, u: f64) -> Result<Geom2dCurveD1, Failure> {
        curve.eval_d1(u)
    }
    fn local(
        curve: &Geom2dBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<Geom2dCurveD1, Failure> {
        let (point, d1) = curve.local_d1(u, start, finish)?;
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn cached(cache: &Cache, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let (point, d1) = cache.d1::<Point2>(u)?;
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn offset(
        u: f64,
        basis: &Geom2dAdaptorCurve,
        offset: f64,
    ) -> Result<Option<Geom2dCurveD1>, Failure> {
        evaluate_d1(u, basis, offset)
    }
    fn failure() -> Failure {
        undefined_derivative("Geom2dAdaptor_Curve::EvalD1: evaluation failed")
    }
}

/// `EvalD2`: a line has no ElCLib D2, its second derivative is set to zero.
pub(super) struct EvalD2;
impl EvalOrder for EvalD2 {
    type Output = Geom2dCurveD2;
    fn elementary(data: &CurveData, u: f64) -> Option<Geom2dCurveD2> {
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
        Some(Geom2dCurveD2 { point, d1, d2 })
    }
    fn curve(curve: &dyn Geom2dCurve, u: f64) -> Result<Geom2dCurveD2, Failure> {
        curve.eval_d2(u)
    }
    fn local(
        curve: &Geom2dBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<Geom2dCurveD2, Failure> {
        let (point, d1, d2) = curve.local_d2(u, start, finish)?;
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn cached(cache: &Cache, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let (point, d1, d2) = cache.d2::<Point2>(u)?;
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn offset(
        u: f64,
        basis: &Geom2dAdaptorCurve,
        offset: f64,
    ) -> Result<Option<Geom2dCurveD2>, Failure> {
        evaluate_d2(u, basis, offset)
    }
    fn failure() -> Failure {
        undefined_derivative("Geom2dAdaptor_Curve::EvalD2: evaluation failed")
    }
}

/// `EvalD3`: a line uses D1 and a parabola D2, the missing derivatives set to zero.
pub(super) struct EvalD3;
impl EvalOrder for EvalD3 {
    type Output = Geom2dCurveD3;
    fn elementary(data: &CurveData, u: f64) -> Option<Geom2dCurveD3> {
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
        Some(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn curve(curve: &dyn Geom2dCurve, u: f64) -> Result<Geom2dCurveD3, Failure> {
        curve.eval_d3(u)
    }
    fn local(
        curve: &Geom2dBSplineCurve,
        u: f64,
        start: i32,
        finish: i32,
    ) -> Result<Geom2dCurveD3, Failure> {
        let (point, d1, d2, d3) = curve.local_d3(u, start, finish)?;
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn cached(cache: &Cache, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let (point, d1, d2, d3) = cache.d3::<Point2>(u)?;
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn offset(
        u: f64,
        basis: &Geom2dAdaptorCurve,
        offset: f64,
    ) -> Result<Option<Geom2dCurveD3>, Failure> {
        evaluate_d3(u, basis, offset)
    }
    fn failure() -> Failure {
        undefined_derivative("Geom2dAdaptor_Curve::EvalD3: evaluation failed")
    }
}
