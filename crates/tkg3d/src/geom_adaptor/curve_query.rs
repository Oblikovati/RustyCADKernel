// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.cxx (Resolution and the curve-kind queries) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Resolution and curve-kind queries of `GeomAdaptor_Curve`.
use super::curve_state::{AdaptorState, CurveData};
use crate::geom::errors::no_such_object;
use crate::geom::{GeomBSplineCurve, GeomBezierCurve, GeomOffsetCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3};
use std::f64::consts::PI;

/// Parametric resolution of the space resolution `r_uv` (`Resolution`): exact for
/// lines, circles and ellipses, the curve's own for Bezier and B-spline curves,
/// `Precision::Parametric` otherwise; `resolution(&state, 1e-7)?`.
pub(super) fn resolution(state: &AdaptorState, r_uv: f64) -> Result<f64, Failure> {
    match &state.data {
        CurveData::Line(_) => Ok(r_uv),
        CurveData::Circle(circle) => {
            let radius: f64 = circle.radius();
            if radius > r_uv / 2.0 {
                return Ok(2.0 * (r_uv / (2.0 * radius)).asin());
            }
            Ok(2.0 * PI)
        }
        CurveData::Ellipse(ellipse) => Ok(r_uv / ellipse.major_radius()),
        CurveData::Bezier(data) => data.curve.resolution(r_uv),
        CurveData::BSpline(data) => data.curve.resolution(r_uv),
        _ => Ok(precision::parametric(r_uv)),
    }
}

/// `Line`; Standard_NoSuchObject for another kind; `line(&state)?`.
pub(super) fn line(state: &AdaptorState) -> Result<Line3, Failure> {
    match state.data {
        CurveData::Line(line) => Ok(line),
        _ => Err(no_such_object(
            "GeomAdaptor_Curve::Line() - curve is not a Line",
        )),
    }
}
/// `Circle`; Standard_NoSuchObject for another kind; `circle(&state)?`.
pub(super) fn circle(state: &AdaptorState) -> Result<Circle3, Failure> {
    match state.data {
        CurveData::Circle(circle) => Ok(circle),
        _ => Err(no_such_object(
            "GeomAdaptor_Curve::Circle() - curve is not a Circle",
        )),
    }
}
/// `Ellipse`; Standard_NoSuchObject for another kind; `ellipse(&state)?`.
pub(super) fn ellipse(state: &AdaptorState) -> Result<Ellipse3, Failure> {
    match state.data {
        CurveData::Ellipse(ellipse) => Ok(ellipse),
        _ => Err(no_such_object(
            "GeomAdaptor_Curve::Ellipse() - curve is not an Ellipse",
        )),
    }
}
/// `Hyperbola`; Standard_NoSuchObject for another kind; `hyperbola(&state)?`.
pub(super) fn hyperbola(state: &AdaptorState) -> Result<Hyperbola3, Failure> {
    match state.data {
        CurveData::Hyperbola(hyperbola) => Ok(hyperbola),
        _ => Err(no_such_object(
            "GeomAdaptor_Curve::Hyperbola() - curve is not a Hyperbola",
        )),
    }
}
/// `Parabola`; Standard_NoSuchObject for another kind; `parabola(&state)?`.
pub(super) fn parabola(state: &AdaptorState) -> Result<Parabola3, Failure> {
    match state.data {
        CurveData::Parabola(parabola) => Ok(parabola),
        _ => Err(no_such_object(
            "GeomAdaptor_Curve::Parabola() - curve is not a Parabola",
        )),
    }
}

fn no_poles(member: &str) -> Failure {
    no_such_object(&format!(
        "GeomAdaptor_Curve::{member}: expected a Bezier or B-spline curve"
    ))
}

/// `Degree` of a Bezier or B-spline curve; `degree(&state)?`.
pub(super) fn degree(state: &AdaptorState) -> Result<i32, Failure> {
    match &state.data {
        CurveData::Bezier(data) => Ok(data.curve.degree()),
        CurveData::BSpline(data) => Ok(data.curve.degree()),
        _ => Err(no_poles("Degree")),
    }
}
/// `IsRational`: false for every other kind; `is_rational(&state)`.
pub(super) fn is_rational(state: &AdaptorState) -> bool {
    match &state.data {
        CurveData::BSpline(data) => data.curve.is_rational(),
        CurveData::Bezier(data) => data.curve.is_rational(),
        _ => false,
    }
}
/// `NbPoles` of a Bezier or B-spline curve; `nb_poles(&state)?`.
pub(super) fn nb_poles(state: &AdaptorState) -> Result<i32, Failure> {
    match &state.data {
        CurveData::Bezier(data) => Ok(data.curve.nb_poles()),
        CurveData::BSpline(data) => Ok(data.curve.nb_poles()),
        _ => Err(no_poles("NbPoles")),
    }
}
/// `NbKnots` of a B-spline curve; `nb_knots(&state)?`.
pub(super) fn nb_knots(state: &AdaptorState) -> Result<i32, Failure> {
    match &state.data {
        CurveData::BSpline(data) => Ok(data.curve.nb_knots()),
        _ => Err(no_such_object("GeomAdaptor_Curve::NbKnots")),
    }
}
/// `Bezier`: the loaded Bezier curve; Standard_NoSuchObject for another kind (unlike
/// Geom2dAdaptor, which returns a null handle); `bezier(&state)?`.
pub(super) fn bezier(state: &AdaptorState) -> Result<Handle<GeomBezierCurve>, Failure> {
    match &state.data {
        CurveData::Bezier(data) => Ok(data.curve.clone()),
        _ => Err(no_such_object("GeomAdaptor_Curve::Bezier")),
    }
}
/// `BSpline`: the loaded B-spline curve; Standard_NoSuchObject for another kind;
/// `bspline(&state)?`.
pub(super) fn bspline(state: &AdaptorState) -> Result<Handle<GeomBSplineCurve>, Failure> {
    match &state.data {
        CurveData::BSpline(data) => Ok(data.curve.clone()),
        _ => Err(no_such_object("GeomAdaptor_Curve::BSpline")),
    }
}
/// `OffsetCurve`: the loaded offset curve; Standard_NoSuchObject for another kind;
/// `offset_curve(&state)?`.
pub(super) fn offset_curve(state: &AdaptorState) -> Result<Handle<GeomOffsetCurve>, Failure> {
    match &state.data {
        CurveData::Offset(_) => Ok(state.curve.cast()),
        _ => Err(no_such_object("GeomAdaptor_Curve::OffsetCurve")),
    }
}
