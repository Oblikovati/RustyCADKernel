//! Express sampling and deep maximum-distance checks from OCCT 3d097a0.
use super::{
    distance::{CurveDistance, DistanceBest, DistanceSample},
    distance_search::find_distance,
    edge_math::split_range,
};
use crate::int_tools::Range;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
pub(super) struct CurvePair<'a> {
    pub first: &'a Handle<dyn GeomCurve>,
    pub second: &'a Handle<dyn GeomCurve>,
    pub first_range: Range,
    pub second_range: Range,
}
impl CurvePair<'_> {
    pub(super) fn projector(&self) -> Result<ProjectPointOnCurve, Failure> {
        let mut projector = ProjectPointOnCurve::new();
        projector.init_curve(
            self.second,
            self.second_range.first(),
            self.second_range.last(),
        )?;
        Ok(projector)
    }
    pub(super) fn is_coincident(&self, tolerance: f64) -> Result<bool, Failure> {
        let mut projector = self.projector()?;
        let step = (self.first_range.last() - self.first_range.first()) / 23.;
        let mut count = 0;
        for index in 0..=23 {
            let parameter = self.first_range.first() + f64::from(index) * step;
            projector.perform(self.first.eval_d0(parameter)?)?;
            if projector.nb_points() > 0 && projector.lower_distance()? < tolerance {
                count += 1;
            }
        }
        Ok(f64::from(count) / 24. > 0.5)
    }
    pub(super) fn check_coincidence(
        &self,
        threshold: f64,
        resolution: f64,
    ) -> Result<i32, Failure> {
        let mut projector = self.projector()?;
        let mut distance = CurveDistance {
            curve: self.first,
            projector: &mut projector,
            threshold,
            sign: 1,
        };
        let mut ranges = Vec::new();
        let count = split_range(
            self.first_range.first(),
            self.first_range.last(),
            resolution,
            10,
            &mut ranges,
        );
        let mut sample = DistanceSample {
            distance: -1.,
            parameter: 0.,
        };
        for range in ranges.iter().take((count - 1).max(0) as usize) {
            let status = distance.evaluate(range.last(), &mut sample)?;
            if status != 0 {
                return Ok(status);
            }
        }
        if count < 10 {
            return Ok(0);
        }
        deep_coincidence(&mut distance, &ranges, count, resolution, sample.distance)
    }
}
fn deep_coincidence(
    distance: &mut CurveDistance<'_>,
    ranges: &[Range],
    count: i32,
    resolution: f64,
    initial: f64,
) -> Result<i32, Failure> {
    let mut best = DistanceBest {
        distance: initial,
        ..DistanceBest::default()
    };
    for range in ranges.iter().take((count - 1) as usize).skip(1) {
        let status = find_distance(
            distance,
            (range.first(), range.last()),
            resolution,
            &mut best,
        )?;
        if status != 0 {
            return Ok(status);
        }
    }
    Ok(0)
}
