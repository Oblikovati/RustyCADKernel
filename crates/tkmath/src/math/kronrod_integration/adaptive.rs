use super::{Function, KronrodSingleIntegration, Quadrature};
use crate::math::real_epsilon;
struct Intervals {
    bounds: Vec<f64>,
    errors: Vec<f64>,
    values: Vec<f64>,
    stagnation: i32,
}
impl Intervals {
    fn largest_error(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut maximum: f64 = 0.0;
        for (index, &error) in self.errors.iter().enumerate() {
            if error > maximum {
                maximum = error;
                best = Some(index);
            }
        }
        best
    }
    fn insert(&mut self, index: usize, center: f64, values: [f64; 2], errors: [f64; 2]) {
        self.bounds.insert(index + 1, center);
        self.values[index] = values[0];
        self.values.insert(index + 1, values[1]);
        self.errors[index] = errors[0];
        self.errors.insert(index + 1, errors[1]);
    }
}
impl KronrodSingleIntegration {
    pub(super) fn refine(
        &mut self,
        f: &mut impl Function,
        q: &Quadrature,
        bounds: [f64; 2],
        tolerance: f64,
        limit: i32,
    ) {
        let mut intervals: Intervals = Intervals {
            bounds: bounds.to_vec(),
            errors: vec![self.absolute],
            values: vec![self.value],
            stagnation: 0,
        };
        while self.error > tolerance && self.iterations < limit {
            let Some(index) = intervals.largest_error() else {
                self.done = false;
                return;
            };
            if !self.subdivide(f, q, &mut intervals, index) {
                return;
            }
        }
    }
    fn subdivide(
        &mut self,
        f: &mut impl Function,
        q: &Quadrature,
        intervals: &mut Intervals,
        index: usize,
    ) -> bool {
        let lower: f64 = intervals.bounds[index];
        let upper: f64 = intervals.bounds[index + 1];
        let center: f64 = 0.5 * (lower + upper);
        let mut values: [f64; 2] = [0.0; 2];
        let mut errors: [f64; 2] = [0.0; 2];
        self.done = q.evaluate(f, [lower, center], &mut values[0], &mut errors[0]);
        if !self.done {
            return false;
        }
        self.done = q.evaluate(f, [center, upper], &mut values[1], &mut errors[1]);
        if !self.done {
            return false;
        }
        self.iterations += 1;
        self.update(intervals, index, values, errors);
        if intervals.stagnation > 50 {
            return false;
        }
        intervals.insert(index, center, values, errors);
        true
    }
    fn update(
        &mut self,
        intervals: &mut Intervals,
        index: usize,
        values: [f64; 2],
        errors: [f64; 2],
    ) {
        let change: f64 = values[0] + values[1] - intervals.values[index];
        self.value += change;
        if change.abs() <= real_epsilon(self.value.abs()) {
            intervals.stagnation += 1;
        }
        let change: f64 = errors[0] + errors[1] - intervals.errors[index];
        self.absolute += change;
        if self.absolute <= real_epsilon(self.value.abs()) {
            intervals.stagnation += 1;
        }
        self.error = if self.value.abs() > real_epsilon(1.0) {
            self.absolute / self.value.abs()
        } else {
            self.absolute
        };
    }
}
