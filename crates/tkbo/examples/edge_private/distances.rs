//! Projection and golden-section state compared through the original private helpers.
use super::super::{
    distance::{CurveDistance, DistanceBest, DistanceSample},
    distance_search::find_distance,
};
use super::{fixtures, integer, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
fn snapshot(sample: &DistanceSample, best: &DistanceBest) {
    for value in [
        sample.distance,
        sample.parameter,
        best.distance,
        best.first,
        best.second,
    ] {
        real(value);
    }
}
pub(super) fn case(values: &[f64]) -> Result<(), Failure> {
    let source = fixtures::geometry(values[0] as i32, 0.5)?;
    let target = fixtures::geometry(values[1] as i32, 0.5)?;
    let mut projector = ProjectPointOnCurve::new();
    projector.init_curve(&target, -2., 3.)?;
    let maximum = values[6] != 0.;
    let mut distance = CurveDistance {
        curve: &source,
        projector: &mut projector,
        threshold: values[4],
        sign: if maximum { 1 } else { -1 },
    };
    let mut sample = DistanceSample {
        distance: 17.,
        parameter: 19.,
    };
    let mut best = DistanceBest {
        distance: if maximum { -1. } else { 1e100 },
        first: 23.,
        second: 29.,
    };
    integer(distance.evaluate((values[2] + values[3]) * 0.5, &mut sample)?);
    snapshot(&sample, &best);
    integer(distance.evaluate_best(values[2], &mut sample, &mut best)?);
    snapshot(&sample, &best);
    integer(find_distance(
        &mut distance,
        (values[2], values[3]),
        values[5],
        &mut best,
    )?);
    snapshot(&sample, &best);
    integer(projector.nb_points());
    if projector.nb_points() > 0 {
        real(projector.lower_distance()?);
        real(projector.lower_distance_parameter()?);
    }
    Ok(())
}
