//! Private IntTools_EdgeEdge scalar helpers from OCCT 3d097a0.
use crate::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, geom::GeomCurve};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    bvh::BvhScalar,
    geom_abs::CurveType,
    gp::{Direction3, Point3, resolution},
};
pub(super) fn curve_priority(kind: CurveType) -> i32 {
    match kind {
        CurveType::Line => 0,
        CurveType::Hyperbola | CurveType::Parabola => 1,
        CurveType::Circle | CurveType::Ellipse => 2,
        CurveType::BezierCurve | CurveType::BSplineCurve => 3,
        _ => 4,
    }
}
pub(super) fn split_range(
    first: f64,
    last: f64,
    resolution: f64,
    requested: i32,
    output: &mut Vec<Range>,
) -> i32 {
    let difference = last - first;
    if difference < resolution || requested == 1 {
        output.push(Range::new(first, last));
        return 1;
    }
    let mut count = requested;
    let mut step = difference / f64::from(count);
    if step < resolution {
        count = (difference / resolution).truncate_to_int().wrapping_add(1);
        step = difference / f64::from(count);
    }
    let mut start = first;
    for _ in 1..count {
        let end = start + step;
        output.push(Range::new(start, end));
        start = end;
    }
    output.push(Range::new(start, last));
    count
}
pub(super) fn point_box_distance(bounds: &BoundingBox3, point: Point3) -> Result<f64, Failure> {
    let limits = bounds.get()?;
    let coordinates = [point.x(), point.y(), point.z()];
    let mut distance = 0.;
    for index in 0..3 {
        let lower = limits[index] - coordinates[index];
        if lower > 0. {
            distance += lower * lower;
            continue;
        }
        let upper = coordinates[index] - limits[index + 3];
        if upper > 0. {
            distance += upper * upper;
        }
    }
    Ok(distance.sqrt())
}
pub(super) fn curve_deflection(curve: &BRepAdaptorCurve, range: Range) -> Result<f64, Failure> {
    let step = (range.last() - range.first()) / 10.;
    let mut parameter = range.first();
    let (_, mut previous) = curve.d1(parameter)?;
    let mut deflection = 0.;
    for _ in 0..10 {
        parameter += step;
        let (_, current) = curve.d1(parameter)?;
        if previous.magnitude() > resolution() && current.magnitude() > resolution() {
            let a = Direction3::new(previous.x(), previous.y(), previous.z())?;
            let b = Direction3::new(current.x(), current.y(), current.z())?;
            deflection += a.angle(&b);
        }
        previous = current;
    }
    Ok(deflection)
}
pub(super) fn is_closed(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
    tolerance: f64,
    resolution: f64,
) -> Result<bool, Failure> {
    if (first - last).abs() < resolution {
        return Ok(false);
    }
    let a = curve.eval_d0(first)?;
    let b = curve.eval_d0(last)?;
    Ok(a.distance(&b) < tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_curve_family_has_the_native_sort_priority() {
        use CurveType::*;
        for (kind, expected) in [
            (Line, 0),
            (Hyperbola, 1),
            (Parabola, 1),
            (Circle, 2),
            (Ellipse, 2),
            (BezierCurve, 3),
            (BSplineCurve, 3),
            (OffsetCurve, 4),
            (OtherCurve, 4),
        ] {
            assert_eq!(curve_priority(kind), expected);
        }
    }
    #[test]
    fn void_distance_reports_the_native_bounding_box_failure() {
        assert!(point_box_distance(&BoundingBox3::default(), Point3::default()).is_err());
    }
}
