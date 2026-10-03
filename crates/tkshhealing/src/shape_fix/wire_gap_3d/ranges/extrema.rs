use super::{Failure, GapPair, RangeCurve, candidates};
use rustycad_tkernel::precision;
use rustycad_tkgeomalgo::geom_api::ExtremaCurveCurve;
use rustycad_tkmath::gp::Point3;
pub(super) fn closest(
    pair: &GapPair,
    first: &RangeCurve<'_>,
    second: &RangeCurve<'_>,
) -> Result<Option<(f64, f64)>, Failure> {
    let extrema = ExtremaCurveCurve::with_ranges(
        &first.repair.curve,
        &second.repair.curve,
        (first.entry.first, first.entry.last),
        (second.entry.first, second.entry.last),
    )?;
    if extrema.nb_extrema() == 0 {
        return Ok(None);
    }
    // Native catches failures only after constructing the extrema object.
    Ok(select_and_validate(pair, first, second, &extrema).unwrap_or(None))
}
fn select_and_validate(
    pair: &GapPair,
    first: &RangeCurve<'_>,
    second: &RangeCurve<'_>,
    extrema: &ExtremaCurveCurve,
) -> Result<Option<(f64, f64)>, Failure> {
    let indices = nearest_intersections(first, second, extrema)?;
    let parameters = if indices.0 != 0 && indices.1 != 0 {
        let index = choose_intersection(pair, first, second, extrema, indices)?;
        extrema.parameters(index)?
    } else {
        extrema.lower_distance_parameters()?
    };
    let parameters = (
        candidates::periodic_parameter(first, parameters.0)?,
        candidates::periodic_parameter(second, parameters.1)?,
    );
    Ok(candidates::acceptable(pair, first, second, parameters)?.then_some(parameters))
}
fn nearest_intersections(
    first: &RangeCurve<'_>,
    second: &RangeCurve<'_>,
    extrema: &ExtremaCurveCurve,
) -> Result<(i32, i32), Failure> {
    let mut indices = (0, 0);
    let mut distances = (-1., -1.);
    for index in 1..=extrema.nb_extrema() {
        let (u, v) = extrema.parameters(index)?;
        let u = candidates::periodic_parameter(first, u)?;
        let v = candidates::periodic_parameter(second, v)?;
        if first
            .repair
            .curve
            .value(u)?
            .distance(&second.repair.curve.value(v)?)
            >= precision::confusion()
        {
            continue;
        }
        let delta = (
            (first.opposite_endpoint() - u).abs(),
            (second.opposite_endpoint() - v).abs(),
        );
        if distances.0 < 0. || distances.0 > delta.0 {
            indices.0 = index;
            distances.0 = delta.0;
        }
        if distances.1 < 0. || distances.1 > delta.1 {
            indices.1 = index;
            distances.1 = delta.1;
        }
    }
    Ok(indices)
}
fn choose_intersection(
    pair: &GapPair,
    first: &RangeCurve<'_>,
    second: &RangeCurve<'_>,
    extrema: &ExtremaCurveCurve,
    indices: (i32, i32),
) -> Result<i32, Failure> {
    if indices.0 == indices.1 {
        return Ok(indices.0);
    }
    let first_point = midpoint(first, second, extrema.parameters(indices.0)?)?;
    let second_point = midpoint(first, second, extrema.parameters(indices.1)?)?;
    Ok(
        if second_point.distance(&pair.target) < first_point.distance(&pair.target) {
            indices.1
        } else {
            indices.0
        },
    )
}
fn midpoint(
    first: &RangeCurve<'_>,
    second: &RangeCurve<'_>,
    parameters: (f64, f64),
) -> Result<Point3, Failure> {
    let a = first.repair.curve.value(parameters.0)?;
    let b = second.repair.curve.value(parameters.1)?;
    Ok(Point3::new(
        (a.x() + b.x()) * 0.5,
        (a.y() + b.y()) * 0.5,
        (a.z() + b.z()) * 0.5,
    ))
}
