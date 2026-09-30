use super::{Failure, GlobalOptimum, MultipleVarFunction, Vector, native_max, native_min};
impl GlobalOptimum<'_> {
    pub(super) fn initial_solutions(&mut self) -> Result<(), Failure> {
        let mut point: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        let mut value: f64 = 0.0;
        for i in 1..=self.dimension {
            point[i] = (self.global_lower[i] + self.global_upper[i]) * 0.5;
        }
        self.function.value(&point, &mut value);
        self.add_candidate(&point, value);
        for sample in 0..3 {
            for i in 1..=self.dimension {
                point[i] =
                    self.lower[i] + ((self.upper[i] - self.lower[i]) * f64::from(sample)) * 0.5;
            }
            let start: Vector<'static> = point.clone();
            if self.local(&start, &mut value, &mut point)? {
                self.add_candidate(&point, value);
            }
        }
        Ok(())
    }
    pub(super) fn estimate_constant(&mut self) -> Result<(), Failure> {
        let mut diagonal: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        for i in 1..=self.dimension {
            diagonal[i] = self.upper[i] - self.lower[i];
        }
        let step: f64 = diagonal.norm()? / 13.0;
        let increments: Vector<'static> = diagonal.divided(13.0)?;
        let mut point: Vector<'static> = diagonal.clone();
        let mut previous: f64 = 0.0;
        let mut current: f64 = f64::MAX;
        self.function.value(&self.lower, &mut previous);
        let mut projected: f64 = previous;
        let mut estimate: f64 = 0.0;
        for sample in 1..=13 {
            for i in 1..=self.dimension {
                point[i] = self.lower[i] + increments[i] * f64::from(sample);
            }
            self.function.value(&point, &mut current);
            estimate = native_max((current - previous).abs(), estimate);
            previous = current;
            point[1] = self.lower[1];
            self.function.value(&point, &mut current);
            estimate = native_max((current - projected).abs(), estimate);
            projected = current;
        }
        self.constant = self.initial_constant;
        estimate *= f64::from(self.dimension).sqrt() / step;
        if estimate < self.constant * 0.1 {
            self.constant = native_max(estimate * 0.1, 0.01);
        } else if estimate > self.constant * 100.0 {
            self.constant = native_min(self.constant * 100.0, 1000.0);
        }
        Ok(())
    }
}
