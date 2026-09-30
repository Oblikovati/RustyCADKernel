//! Native projection distance comparisons preserve caller-owned output state on failure.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
#[derive(Clone, Copy, Default)]
pub(super) struct DistanceSample {
    pub distance: f64,
    pub parameter: f64,
}
#[derive(Clone, Copy, Default)]
pub(super) struct DistanceBest {
    pub distance: f64,
    pub first: f64,
    pub second: f64,
}
pub(super) struct CurveDistance<'a> {
    pub curve: &'a Handle<dyn GeomCurve>,
    pub projector: &'a mut ProjectPointOnCurve,
    pub threshold: f64,
    pub sign: i32,
}
impl CurveDistance<'_> {
    pub fn evaluate(
        &mut self,
        parameter: f64,
        sample: &mut DistanceSample,
    ) -> Result<i32, Failure> {
        let point = self.curve.eval_d0(parameter)?;
        self.projector.perform(point)?;
        if self.projector.nb_points() == 0 {
            return Ok(1);
        }
        sample.distance = self.projector.lower_distance()?;
        sample.parameter = self.projector.lower_distance_parameter()?;
        if f64::from(self.sign) * (sample.distance - self.threshold) > 0. {
            return Ok(2);
        }
        Ok(0)
    }
    pub fn evaluate_best(
        &mut self,
        parameter: f64,
        sample: &mut DistanceSample,
        best: &mut DistanceBest,
    ) -> Result<i32, Failure> {
        let status = self.evaluate(parameter, sample)?;
        if status == 1 {
            return Ok(status);
        }
        if f64::from(self.sign) * (sample.distance - best.distance) > 0. {
            best.distance = sample.distance;
            best.first = parameter;
            best.second = sample.parameter;
        }
        Ok(status)
    }
}
