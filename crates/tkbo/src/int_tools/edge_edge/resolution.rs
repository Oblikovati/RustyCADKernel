//! Pinned edge resolution uses repeated parameter addition and signed offset radii.
use crate::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{
        GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomOffsetCurve,
    },
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkmath::geom_abs::CurveType;
pub(super) fn resolution_coefficient(
    curve: &BRepAdaptorCurve,
    range: Range,
) -> Result<f64, Failure> {
    let geometry = curve.transformed_curve().geom_curve();
    match curve.get_type()? {
        CurveType::Circle => Ok(1. / (2. * geometry.cast::<GeomCircle>().circ()?.radius())),
        CurveType::Ellipse => Ok(1. / geometry.cast::<GeomEllipse>().major_radius()),
        CurveType::OffsetCurve => match offset_coefficient(&geometry)? {
            Some(value) => Ok(value),
            None => sampled_coefficient(curve, range),
        },
        CurveType::Hyperbola | CurveType::Parabola | CurveType::OtherCurve => {
            sampled_coefficient(curve, range)
        }
        _ => Ok(0.),
    }
}
fn offset_coefficient(curve: &Handle<dyn GeomCurve>) -> Result<Option<f64>, Failure> {
    let offset = curve.cast::<GeomOffsetCurve>();
    let basis = GeomAdaptorCurve::with_curve(&offset.basis_curve())?;
    Ok(match basis.get_type()? {
        CurveType::Line => Some(0.),
        CurveType::Circle => Some(1. / (2. * (offset.offset() + basis.circle()?.radius()))),
        CurveType::Ellipse => Some(1. / (offset.offset() + basis.ellipse()?.major_radius())),
        _ => None,
    })
}
fn sampled_coefficient(curve: &BRepAdaptorCurve, range: Range) -> Result<f64, Failure> {
    let step = (range.last() - range.first()) / 30.;
    let mut parameter = range.first();
    let mut minimum = 10.;
    let mut previous = curve.value(parameter)?;
    for _ in 0..30 {
        parameter += step;
        let current = curve.value(parameter)?;
        let ratio = step / previous.distance(&current);
        if ratio < minimum {
            minimum = ratio;
        }
        previous = current;
    }
    Ok(minimum)
}
pub(super) fn curve_resolution(
    curve: &Handle<dyn GeomCurve>,
    kind: CurveType,
    coefficient: f64,
    tolerance: f64,
) -> Result<f64, Failure> {
    match kind {
        CurveType::Line => Ok(tolerance),
        CurveType::Circle => Ok(circle_resolution(coefficient, tolerance)),
        CurveType::BezierCurve => curve.cast::<GeomBezierCurve>().resolution(tolerance),
        CurveType::BSplineCurve => curve.cast::<GeomBSplineCurve>().resolution(tolerance),
        CurveType::OffsetCurve => offset_resolution(curve, coefficient, tolerance),
        _ => Ok(coefficient * tolerance),
    }
}
fn circle_resolution(coefficient: f64, tolerance: f64) -> f64 {
    let step = coefficient * tolerance;
    if step <= 1. {
        2. * step.asin()
    } else {
        2. * std::f64::consts::PI
    }
}
fn offset_resolution(
    curve: &Handle<dyn GeomCurve>,
    coefficient: f64,
    tolerance: f64,
) -> Result<f64, Failure> {
    let basis = curve.cast::<GeomOffsetCurve>().basis_curve();
    let adaptor = GeomAdaptorCurve::with_curve(&basis)?;
    Ok(match adaptor.get_type()? {
        CurveType::Line => tolerance,
        CurveType::Circle => circle_resolution(coefficient, tolerance),
        _ => coefficient * tolerance,
    })
}
