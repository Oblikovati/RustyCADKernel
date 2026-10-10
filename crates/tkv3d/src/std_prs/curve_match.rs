//! Match the native sampled polygon, including its circle overshoot and Manhattan metric.
use crate::prs3d::Prs3d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::{geom_abs::CurveType, gp::Point3};

pub(super) fn match_curve(
    query: Point3,
    distance: f64,
    curve: &dyn Adaptor3dCurve,
    deflection: f64,
    count: i32,
    bounds: [f64; 2],
) -> Result<bool, Failure> {
    if curve.get_type()? == CurveType::Line {
        let first = curve.value(bounds[0])?;
        if near_point(query, first, distance) {
            return Ok(true);
        }
        let last = curve.value(bounds[1])?;
        return Ok(near_point(query, last, distance) || near_segment(query, first, last, distance));
    }
    let (count, step) = match_sampling(curve, deflection, count, bounds)?;
    match_samples(query, distance, curve, bounds[0], count, step)
}
fn match_sampling(
    curve: &dyn Adaptor3dCurve,
    deflection: f64,
    count: i32,
    bounds: [f64; 2],
) -> Result<(i32, f64), Failure> {
    if curve.get_type()? == CurveType::Circle {
        let step = (8. * deflection / curve.circle()?.radius()).sqrt();
        let intervals = (((bounds[1] - bounds[0]).abs() / step).trunc() as i32).max(2);
        return Ok((intervals + 1, step));
    }
    Ok((count, (bounds[1] - bounds[0]) / f64::from(count - 1)))
}
fn match_samples(
    query: Point3,
    distance: f64,
    curve: &dyn Adaptor3dCurve,
    first: f64,
    count: i32,
    step: f64,
) -> Result<bool, Failure> {
    let mut previous: Option<Point3> = None;
    for index in 0..count {
        let point = curve.value(first + f64::from(index) * step)?;
        if near_point(query, point, distance)
            || previous.is_some_and(|p| near_segment(query, p, point, distance))
        {
            return Ok(true);
        }
        previous = Some(point);
    }
    Ok(false)
}
fn near_point(query: Point3, point: Point3, distance: f64) -> bool {
    let delta = query.xyz() - point.xyz();
    delta.x().abs() + delta.y().abs() + delta.z().abs() <= distance
}
fn near_segment(query: Point3, first: Point3, last: Point3, distance: f64) -> bool {
    Prs3d::match_segment(query.x(), query.y(), query.z(), distance, &first, &last).0
}
