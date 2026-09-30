use super::{Failure, GlobalOptimum, MultipleVarFunction, Vector, native_max, native_min};
impl GlobalOptimum<'_> {
    /// Searches the local box and retains native candidate state; `solver.perform(false)?`.
    pub fn perform(&mut self, single: bool) -> Result<(), Failure> {
        self.done = false;
        let (mut minimum, mut maximum): (f64, f64) = (f64::MAX, -f64::MAX);
        for i in 1..=self.dimension {
            let length: f64 = self.upper[i] - self.lower[i];
            if length < minimum {
                minimum = length;
            }
            if length > maximum {
                maximum = length;
            }
            self.steps[i] = 0.0;
        }
        if minimum < 1e-9 {
            return Ok(());
        }
        if !self.locked {
            self.estimate_constant()?;
        }
        self.epsilon1 = minimum * self.tolerance;
        self.epsilon2 = maximum * self.tolerance;
        self.single = single;
        self.epsilon3 = if single {
            0.0
        } else if self.constant > 1.0 {
            -maximum * self.tolerance / 4.0
        } else {
            -maximum * self.tolerance * self.constant / 4.0
        };
        if self.stop() {
            self.done = true;
            return Ok(());
        }
        self.last_step = 0.0;
        self.search_dimension(self.dimension)?;
        self.done = true;
        Ok(())
    }
    fn search_dimension(&mut self, dimension: i32) -> Result<(), Failure> {
        let mut value: f64 = f64::MAX;
        let mut local_value: f64 = f64::MAX;
        let mut reached: bool = false;
        self.point[dimension] = self.lower[dimension] + self.epsilon1;
        while !reached {
            if self.point[dimension] > self.upper[dimension] {
                self.point[dimension] = self.upper[dimension];
                reached = true;
            }
            if self.stop() {
                return Ok(());
            }
            if dimension == 1 {
                self.search_line(&mut value, &mut local_value)?;
                if self.stop() {
                    return Ok(());
                }
            } else {
                self.steps[dimension] = f64::MAX / 2.0;
                self.search_dimension(dimension - 1)?;
                for i in 1..dimension {
                    self.steps[i] = 0.0;
                }
            }
            self.propagate_step(dimension);
            self.point[dimension] += self.steps[dimension];
        }
        Ok(())
    }
    fn search_line(&mut self, value: &mut f64, local_value: &mut f64) -> Result<(), Failure> {
        let previous: f64 = *value;
        self.function.value(&self.point, value);
        let first: f64 = -(*value + -self.constant * self.last_step - self.best);
        let second: f64 = -((*value + previous - self.constant * self.last_step) * 0.5 - self.best);
        let mut inside: bool = false;
        if native_min(first, second) > self.epsilon3 && self.should_refine(*value, previous) {
            let start: Vector<'static> = self.point.clone();
            let mut output: Vector<'static> = self.temporary.clone();
            inside = self.local(&start, local_value, &mut output)?;
            self.temporary = output;
        }
        let better: bool = inside && *local_value < *value;
        let point: Vector<'static> = if better {
            self.temporary.clone()
        } else {
            self.point.clone()
        };
        self.add_candidate(&point, if better { *local_value } else { *value });
        if self.stop() {
            return Ok(());
        }
        self.steps[1] = native_min(
            self.epsilon2 + (self.best - *value).abs() / self.constant,
            self.max_steps[1],
        );
        self.last_step = self.steps[1];
        Ok(())
    }
    fn should_refine(&mut self, value: f64, previous: f64) -> bool {
        let saved: f64 = self.point[1];
        let mut parameter: f64 =
            (2.0 * self.point[1] - self.steps[1]) * 0.5 + (previous - value) * 0.5 / self.constant;
        if previous.abs() >= 1e100 {
            parameter = self.point[1] - self.steps[1] * 0.5;
        }
        self.point[1] = parameter;
        let mut midpoint_value: f64 = 0.0;
        self.function.value(&self.point, &mut midpoint_value);
        self.point[1] = saved;
        (midpoint_value < value && midpoint_value < previous)
            || self.border_distance() < self.epsilon1
    }
    fn border_distance(&self) -> f64 {
        let mut distance: f64 = f64::MAX;
        for i in 1..=self.dimension {
            distance = native_min(
                distance,
                native_min(
                    (self.point[i] - self.lower[i]).abs(),
                    (self.point[i] - self.upper[i]).abs(),
                ),
            );
        }
        distance
    }
    fn propagate_step(&mut self, dimension: i32) {
        if dimension >= self.dimension {
            return;
        }
        let step: f64 = native_max(self.steps[dimension], self.epsilon2);
        if self.steps[dimension + 1] > step {
            self.steps[dimension + 1] = if step > self.max_steps[dimension + 1] {
                self.max_steps[dimension + 1]
            } else {
                step
            };
        }
    }
}
