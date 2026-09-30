//! Golden-section projection search, including the native mandatory first refinement.
use super::distance::{CurveDistance, DistanceBest, DistanceSample};
use rustycad_tkernel::standard::{Failure, real::epsilon};
use rustycad_tkg2d::geom2d::native::std_max;
const GOLDEN_SECTION: f64 = 0.618_033_988_749_894_9;
struct SearchState {
    first: f64,
    last: f64,
    right: f64,
    left: f64,
    right_sample: DistanceSample,
    left_sample: DistanceSample,
}
pub(super) fn find_distance(
    distance: &mut CurveDistance<'_>,
    range: (f64, f64),
    precision: f64,
    best: &mut DistanceBest,
) -> Result<i32, Failure> {
    best.first = 0.;
    best.second = 0.;
    let mut state = SearchState::new(range);
    let status = state.initialize(distance, best)?;
    if status != 0 {
        return Ok(status);
    }
    let magnitude = std_max(state.first.abs(), state.last.abs());
    let tolerance = std_max(precision, epsilon(magnitude) * 10.);
    state.refine(distance, best, tolerance)
}
impl SearchState {
    fn new((first, last): (f64, f64)) -> Self {
        Self {
            first,
            last,
            right: 0.,
            left: 0.,
            right_sample: DistanceSample::default(),
            left_sample: DistanceSample::default(),
        }
    }
    fn initialize(
        &mut self,
        distance: &mut CurveDistance<'_>,
        best: &mut DistanceBest,
    ) -> Result<i32, Failure> {
        let status = distance.evaluate_best(self.first, &mut self.right_sample, best)?;
        if status == 2 {
            return Ok(status);
        }
        let status = distance.evaluate_best(self.last, &mut self.left_sample, best)?;
        if status == 2 {
            return Ok(status);
        }
        self.right = self.first + (self.last - self.first) * GOLDEN_SECTION;
        self.left = self.last - (self.last - self.first) * GOLDEN_SECTION;
        let status = distance.evaluate_best(self.right, &mut self.right_sample, best)?;
        if status != 0 {
            return Ok(status);
        }
        distance.evaluate_best(self.left, &mut self.left_sample, best)
    }
    fn refine(
        &mut self,
        distance: &mut CurveDistance<'_>,
        best: &mut DistanceBest,
        tolerance: f64,
    ) -> Result<i32, Failure> {
        loop {
            let status = self.step(distance, best)?;
            if status != 0 {
                if status == 2 && distance.sign == -1 {
                    self.right = (self.first + self.last) * 0.5;
                    distance.evaluate_best(self.right, &mut self.right_sample, best)?;
                }
                return Ok(status);
            }
            if self.last - self.first < tolerance {
                return Ok(status);
            }
        }
    }
    fn step(
        &mut self,
        distance: &mut CurveDistance<'_>,
        best: &mut DistanceBest,
    ) -> Result<i32, Failure> {
        if f64::from(distance.sign) * (self.right_sample.distance - self.left_sample.distance) > 0.
        {
            self.first = self.left;
            self.left = self.right;
            self.left_sample.distance = self.right_sample.distance;
            self.right = self.first + (self.last - self.first) * GOLDEN_SECTION;
            return distance.evaluate_best(self.right, &mut self.right_sample, best);
        }
        self.last = self.right;
        self.right = self.left;
        self.right_sample.distance = self.left_sample.distance;
        self.left = self.last - (self.last - self.first) * GOLDEN_SECTION;
        distance.evaluate_best(self.left, &mut self.left_sample, best)
    }
}
