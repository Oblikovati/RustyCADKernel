use super::{Failure, GapPair, RangeCurve, is_ellipse_or_circle};
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkernel::precision;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomHyperbola, GeomLine, GeomParabola,
};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Point3;
pub(super) fn project(entry: &RangeCurve<'_>, target: Point3) -> Result<f64, Failure> {
    let domain = projection_domain(entry);
    let projection =
        ProjectPointOnCurve::with_range(target, &entry.repair.curve, domain.0, domain.1)?;
    let mut parameter = entry.endpoint();
    let mut distance = -1.;
    for index in 1..=projection.nb_points() {
        let next = target.distance(&projection.point(index)?);
        if distance < 0. || distance > next {
            distance = next;
            parameter = projection.parameter(index)?;
        }
    }
    periodic_parameter(entry, parameter)
}
fn projection_domain(entry: &RangeCurve<'_>) -> (f64, f64) {
    let curve = &entry.repair.curve;
    if !curve.cast::<GeomBSplineCurve>().is_null() || !curve.cast::<GeomBezierCurve>().is_null() {
        return (curve.first_parameter(), curve.last_parameter());
    }
    if is_ellipse_or_circle(curve) {
        return (0., 2. * std::f64::consts::PI);
    }
    let (mut first, mut last) = (entry.entry.first, entry.entry.last);
    if !curve.cast::<GeomLine>().is_null()
        || !curve.cast::<GeomParabola>().is_null()
        || !curve.cast::<GeomHyperbola>().is_null()
    {
        let extension = 10. * (last - first);
        if entry.first_end {
            first -= extension;
        } else {
            last += extension;
        }
    }
    (first, last)
}
pub(super) fn periodic_parameter(entry: &RangeCurve<'_>, parameter: f64) -> Result<f64, Failure> {
    let curve = &entry.repair.curve;
    if !ShapeAnalysisCurve::is_periodic(curve)? {
        return Ok(parameter);
    }
    let period = curve.period()?;
    let first = entry.entry.first;
    let last = entry.entry.last;
    let mut shift = -(first / period).trunc() * period;
    if first < 0. {
        shift += period;
    }
    if entry.first_end && parameter > last + shift && parameter > first + shift {
        return Ok(parameter - period - shift);
    }
    if !entry.first_end && parameter < last + shift && parameter < first + shift {
        return Ok(parameter + period - shift);
    }
    Ok(parameter)
}
pub(super) fn acceptable(
    pair: &GapPair,
    first: &RangeCurve<'_>,
    second: &RangeCurve<'_>,
    parameters: (f64, f64),
) -> Result<bool, Failure> {
    let (u, v) = parameters;
    let p = first.repair.curve.value(u)?;
    let q = second.repair.curve.value(v)?;
    let gap = pair.first.point.distance(&pair.second.point);
    Ok(p.distance(&q) <= gap
        && (first.opposite_endpoint() - u).abs() > precision::p_confusion()
        && (second.opposite_endpoint() - v).abs() > precision::p_confusion()
        && ((u > first.entry.first && u < first.entry.last)
            || (v > second.entry.first && v < second.entry.last)
            || pair.first.point.distance(&p) <= gap
            || pair.second.point.distance(&q) <= gap))
}
