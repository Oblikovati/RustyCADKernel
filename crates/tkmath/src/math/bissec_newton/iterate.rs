use super::{BissecNewton, FunctionWithDerivative, Status};
impl BissecNewton {
    pub(super) fn iterate<F: FunctionWithDerivative>(
        &mut self,
        function: &mut F,
        [mut low, mut high]: [f64; 2],
        limit: i32,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) {
        let mut old_step: f64 = self.dx;
        for _ in 1..=limit {
            if self.step(low, high, &mut old_step) || criterion(self, function) {
                self.succeed();
                return;
            }
            if !function.values(self.x, &mut self.value, &mut self.derivative) {
                self.status = Status::FunctionError;
                return;
            }
            if self.update_bracket(&mut low, &mut high) {
                self.succeed();
                return;
            }
        }
        self.status = Status::TooManyIterations;
    }
    fn succeed(&mut self) {
        self.status = Status::Ok;
        self.done = true;
    }
    fn step(&mut self, low: f64, high: f64, old_step: &mut f64) -> bool {
        let outside: bool = ((self.x - high) * self.derivative - self.value)
            * ((self.x - low) * self.derivative - self.value)
            >= 0.0;
        let slow: bool = (2.0 * self.value).abs() > (*old_step * self.derivative).abs();
        *old_step = self.dx;
        if outside || slow {
            self.dx = 0.5 * (high - low);
            self.x = low + self.dx;
            return self.dx.abs() < self.tolerance;
        }
        self.dx = self.value / self.derivative;
        let previous: f64 = self.x;
        self.x -= self.dx;
        previous == self.x
    }
    fn update_bracket(&self, low: &mut f64, high: &mut f64) -> bool {
        if self.value < 0.0 {
            *low = self.x;
            return false;
        }
        if self.value > 0.0 {
            *high = self.x;
            return false;
        }
        true
    }
}
