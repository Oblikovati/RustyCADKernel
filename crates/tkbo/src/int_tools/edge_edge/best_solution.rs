//! Search all subranges and retain the native midpoint rule for repeated tangencies.
use super::{
    distance::{CurveDistance, DistanceBest, DistanceSample},
    distance_search::find_distance,
    edge_coincidence::CurvePair,
    edge_math::split_range,
    edge_resolution::curve_resolution,
};
use crate::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
struct TouchState {
    first_range: Range,
    second_range: Range,
    touched: bool,
    confirmed: bool,
    found: bool,
    minimum: f64,
}
pub(super) fn best_solution(
    pair: &CurvePair<'_>,
    curve: &BRepAdaptorCurve,
    coefficient: f64,
    tolerance: f64,
    precision: f64,
    output: &mut (f64, f64),
) -> Result<(), Failure> {
    let resolution = curve_resolution(
        &curve.transformed_curve().geom_curve(),
        curve.get_type()?,
        coefficient,
        tolerance,
    )?;
    let mut ranges = Vec::new();
    split_range(
        pair.first_range.first(),
        pair.first_range.last(),
        3. * resolution,
        10,
        &mut ranges,
    );
    let mut projector = pair.projector()?;
    let mut distance = CurveDistance {
        curve: pair.first,
        projector: &mut projector,
        threshold: 5e-16,
        sign: -1,
    };
    let mut state = TouchState::new(pair);
    for range in ranges {
        let mut best = DistanceBest {
            distance: tolerance,
            ..DistanceBest::default()
        };
        if find_distance(
            &mut distance,
            (range.first(), range.last()),
            precision,
            &mut best,
        )? != 1
        {
            state.update(best, output);
        }
    }
    if !state.found || state.confirmed {
        state.midpoint(&mut distance, output)?;
    }
    Ok(())
}
impl TouchState {
    fn new(pair: &CurvePair<'_>) -> Self {
        Self {
            first_range: pair.first_range,
            second_range: pair.second_range,
            touched: false,
            confirmed: false,
            found: false,
            minimum: precision::infinite(),
        }
    }
    fn update(&mut self, best: DistanceBest, output: &mut (f64, f64)) {
        if best.distance < self.minimum {
            *output = (best.first, best.second);
            self.minimum = best.distance;
            self.found = true;
        }
        if best.distance < 5e-13 {
            self.touch(best.first, best.second);
        }
    }
    fn touch(&mut self, first: f64, second: f64) {
        if self.touched {
            self.first_range = Range::new(self.first_range.first(), first);
            self.second_range = Range::new(self.second_range.first(), second);
            self.confirmed = true;
        } else {
            self.first_range = Range::new(first, self.first_range.last());
            self.second_range = Range::new(second, self.second_range.last());
            self.touched = true;
        }
    }
    fn midpoint(
        &self,
        distance: &mut CurveDistance<'_>,
        output: &mut (f64, f64),
    ) -> Result<(), Failure> {
        output.0 = (self.first_range.first() + self.first_range.last()) * 0.5;
        let mut sample = DistanceSample {
            parameter: output.1,
            ..DistanceSample::default()
        };
        let status = distance.evaluate(output.0, &mut sample)?;
        output.1 = if status == 1 {
            (self.second_range.first() + self.second_range.last()) * 0.5
        } else {
            sample.parameter
        };
        Ok(())
    }
}
