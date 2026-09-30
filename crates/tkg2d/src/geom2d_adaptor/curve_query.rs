// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.cxx (Resolution, the curve-kind queries, NbSamples) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Resolution, curve-kind queries and sample count of `Geom2dAdaptor_Curve`.
use super::curve_state::{AdaptorState, CurveData};
use crate::geom2d::errors::no_such_object;
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve,
    Geom2dTrimmedCurve,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};
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
pub(super) fn line(state: &AdaptorState) -> Result<Line2, Failure> {
    match state.data {
        CurveData::Line(line) => Ok(line),
        _ => Err(no_such_object(
            "Geom2dAdaptor_Curve::Line() - curve is not a Line",
        )),
    }
}
/// `Circle`; Standard_NoSuchObject for another kind; `circle(&state)?`.
pub(super) fn circle(state: &AdaptorState) -> Result<Circle2, Failure> {
    match state.data {
        CurveData::Circle(circle) => Ok(circle),
        _ => Err(no_such_object(
            "Geom2dAdaptor_Curve::Circle() - curve is not a Circle",
        )),
    }
}
/// `Ellipse`; Standard_NoSuchObject for another kind; `ellipse(&state)?`.
pub(super) fn ellipse(state: &AdaptorState) -> Result<Ellipse2, Failure> {
    match state.data {
        CurveData::Ellipse(ellipse) => Ok(ellipse),
        _ => Err(no_such_object(
            "Geom2dAdaptor_Curve::Ellipse() - curve is not an Ellipse",
        )),
    }
}
/// `Hyperbola`; Standard_NoSuchObject for another kind; `hyperbola(&state)?`.
pub(super) fn hyperbola(state: &AdaptorState) -> Result<Hyperbola2, Failure> {
    match state.data {
        CurveData::Hyperbola(hyperbola) => Ok(hyperbola),
        _ => Err(no_such_object(
            "Geom2dAdaptor_Curve::Hyperbola() - curve is not a Hyperbola",
        )),
    }
}
/// `Parabola`; Standard_NoSuchObject for another kind; `parabola(&state)?`.
pub(super) fn parabola(state: &AdaptorState) -> Result<Parabola2, Failure> {
    match state.data {
        CurveData::Parabola(parabola) => Ok(parabola),
        _ => Err(no_such_object(
            "Geom2dAdaptor_Curve::Parabola() - curve is not a Parabola",
        )),
    }
}

fn no_poles(member: &str) -> Failure {
    no_such_object(&format!(
        "Geom2dAdaptor_Curve::{member}: expected a Bezier or B-spline curve"
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
        _ => Err(no_such_object("Geom2dAdaptor_Curve::NbKnots")),
    }
}
/// `BSpline`: the loaded B-spline, or a null handle; `bspline(&state)`.
pub(super) fn bspline(state: &AdaptorState) -> Handle<Geom2dBSplineCurve> {
    match &state.data {
        CurveData::BSpline(data) => data.curve.clone(),
        _ => Handle::default(),
    }
}

fn is_kind<T: RuntimeTyped + ?Sized>(curve: &dyn Geom2dCurve) -> bool {
    curve.dynamic_type().is_subtype(T::static_type())
}

/// `NbSamples` (the file-local `nbPoints`): 2 for a line, `3 + NbPoles` for a Bezier
/// curve, `NbKnots * Degree` (at least 2) for a B-spline, 20 otherwise, capped at 300;
/// offset and trimmed curves take at least 20 or their basis count, without the cap;
/// `nb_samples(&*curve)?`.
pub(super) fn nb_samples(curve: &dyn Geom2dCurve) -> Result<i32, Failure> {
    let mut count: i32 = 20;
    if is_kind::<Geom2dLine>(curve) {
        count = 2;
    } else if is_kind::<Geom2dBezierCurve>(curve) {
        count = 3 + downcast::<Geom2dBezierCurve>(curve)?.nb_poles();
    } else if is_kind::<Geom2dBSplineCurve>(curve) {
        let bspline: &Geom2dBSplineCurve = downcast(curve)?;
        count = (bspline.nb_knots() * bspline.degree()).max(2);
    } else if is_kind::<Geom2dOffsetCurve>(curve) {
        let basis: Handle<dyn Geom2dCurve> = downcast::<Geom2dOffsetCurve>(curve)?.basis_curve();
        return Ok(count.max(nb_samples(&*basis)?));
    } else if is_kind::<Geom2dTrimmedCurve>(curve) {
        let basis: Handle<dyn Geom2dCurve> = downcast::<Geom2dTrimmedCurve>(curve)?.basis_curve();
        return Ok(count.max(nb_samples(&*basis)?));
    }
    Ok(count.min(300))
}

fn downcast<T: RuntimeTyped>(curve: &dyn Geom2dCurve) -> Result<&T, Failure> {
    T::project_from(curve).ok_or_else(|| {
        no_such_object(&format!(
            "Geom2dAdaptor_Curve::NbSamples: {} is not a {}",
            curve.dynamic_type().name(),
            T::static_type().name()
        ))
    })
}
