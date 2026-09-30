use super::{FunctionSetRoot, FunctionSetWithDerivatives};
impl FunctionSetRoot {
    pub(super) fn standard_descent(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
    ) -> bool {
        while self.run.objective / self.run.previous_minimum > 0.005 && !self.run.stop {
            if !self.standard_trial(function) {
                return false;
            }
            self.saved_gradient.initialized(&self.gradient).unwrap();
            if !self.evaluate_or_finish(function) {
                return false;
            }
            self.run.slope = self.gradient.dot_unchecked(&self.direction);
            if self.run.slope.abs() <= 1e-32 {
                if self.run.objective > self.run.old_objective {
                    self.sol.initialized(&self.saved).unwrap();
                }
                self.finish(function, true, true);
                return false;
            }
            if self.run.descent >= 100 {
                self.run.stop = true;
            }
        }
        true
    }
    fn standard_trial(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        if self.run.objective < self.run.old_objective && self.run.slope < 0.0 {
            self.run.descent += 1;
            self.saved.initialized(&self.sol).unwrap();
            self.run.old_objective = self.run.objective;
            self.advance();
            if self.check_divergence() {
                return false;
            }
            self.run.stop = self.trim_bounds();
            self.run.lambda *= 1.7;
            return true;
        }
        if self.run.objective >= self.run.old_objective
            || self.run.objective >= self.run.previous_minimum
        {
            if !self.repair_standard(function) {
                return false;
            }
            self.run.outside = false;
        }
        self.run.stop = true;
        true
    }
    fn repair_standard(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        self.run.good = false;
        if self.run.descent == 0 {
            self.run.descent += 1;
            self.run.good = self.minimize_two(function);
        } else if self.run.change
            || self.run.descent > 1
            || self.run.old_objective > self.run.previous_minimum
        {
            self.run.descent += 1;
            self.run.good = self.minimize_three(function);
        }
        if !self.run.good {
            self.sol.initialized(&self.saved).unwrap();
            self.run.objective = self.run.old_objective;
            return true;
        }
        self.sol
            .initialized(&self.saved.added(&self.delta).unwrap())
            .unwrap();
        if self.check_divergence() {
            return false;
        }
        self.run.outside = self.trim_bounds();
        true
    }
}
