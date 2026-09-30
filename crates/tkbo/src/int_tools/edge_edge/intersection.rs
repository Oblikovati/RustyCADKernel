//! Nearly coincident ranges use endpoint distances, tangents, then a minimum search.
use super::{
    distance::{CurveDistance, DistanceBest},
    distance_search::find_distance,
    edge_coincidence::CurvePair,
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::gp::Vector3;
use std::f64::consts::PI;
pub(super) fn is_intersection(
    pair: &CurvePair<'_>,
    tolerance: f64,
    resolutions: (f64, f64),
) -> Result<bool, Failure> {
    let coefficient = criteria_coefficient(pair, resolutions);
    let threshold = (coefficient * tolerance) * (coefficient * tolerance);
    let [a, b, c, d] = endpoints(pair)?;
    let (p11, p12, p21, p22) = (a.point, b.point, c.point, d.point);
    let (v11, v12, v21, v22) = (a.d1, b.d1, c.d1, d.d1);
    let aligned = p11.square_distance(&p21) < threshold && p12.square_distance(&p22) < threshold;
    let reversed = p11.square_distance(&p22) < threshold && p12.square_distance(&p21) < threshold;
    if !aligned && !reversed || coefficient == 1. {
        return Ok(true);
    }
    let (first, second) = endpoint_angles([v11, v12, v21, v22], aligned)?;
    if !(first < 5e-3 || PI - first < 5e-3 || second < 5e-3 || PI - second < 5e-3) {
        return Ok(true);
    }
    minimum_intersection(pair, tolerance, resolutions.0)
}
fn criteria_coefficient(pair: &CurvePair<'_>, resolutions: (f64, f64)) -> f64 {
    let first = pair.first_range.last() - pair.first_range.first();
    let second = pair.second_range.last() - pair.second_range.first();
    if first > 1e5 * resolutions.0 && second > 1e5 * resolutions.1 {
        return 5000.;
    }
    let ratio = std_min(first / resolutions.0, second / resolutions.1) / 100.;
    if ratio < 1. { 1. } else { ratio }
}
fn endpoints(pair: &CurvePair<'_>) -> Result<[rustycad_tkg3d::geom::GeomCurveD1; 4], Failure> {
    Ok([
        pair.first.eval_d1(pair.first_range.first())?,
        pair.first.eval_d1(pair.first_range.last())?,
        pair.second.eval_d1(pair.second_range.first())?,
        pair.second.eval_d1(pair.second_range.last())?,
    ])
}
fn endpoint_angles(vectors: [Vector3; 4], aligned: bool) -> Result<(f64, f64), Failure> {
    if !vectors
        .iter()
        .all(|vector| vector.square_magnitude() > precision::square_confusion())
    {
        return Ok((0., 0.));
    }
    let [first, last, other_first, other_last] = vectors;
    if aligned {
        Ok((first.angle(other_first)?, last.angle(other_last)?))
    } else {
        Ok((first.angle(other_last)?, last.angle(other_first)?))
    }
}
fn minimum_intersection(
    pair: &CurvePair<'_>,
    tolerance: f64,
    resolution: f64,
) -> Result<bool, Failure> {
    let mut projector = pair.projector()?;
    let mut distance = CurveDistance {
        curve: pair.first,
        projector: &mut projector,
        threshold: tolerance,
        sign: -1,
    };
    let mut best = DistanceBest {
        distance: precision::infinite(),
        ..DistanceBest::default()
    };
    Ok(find_distance(
        &mut distance,
        (pair.first_range.first(), pair.first_range.last()),
        resolution,
        &mut best,
    )? == 2)
}
