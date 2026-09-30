use super::{FunctionSetRoot, FunctionSetWithDerivatives};
impl FunctionSetRoot {
    pub(super) fn stopping<F: FunctionSetWithDerivatives>(
        &mut self,
        function: &mut F,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> bool {
        self.history[self.iteration as usize] = self.run.objective;
        let verify: bool = self.should_verify();
        if verify && self.step_is_small(function, criterion) {
            if self.run.previous_minimum < self.run.objective {
                self.sol.initialized(&self.saved).unwrap();
            }
            self.finish(function, true, true);
            return false;
        }
        if self.run.objective - self.run.previous_minimum <= self.run.function_tolerance {
            return self.analyze_progress(function, verify, criterion);
        }
        if self.run.change {
            self.finish(function, false, false);
            return false;
        }
        self.run.change = true;
        self.sol.initialized(&self.previous).unwrap();
        if !self.evaluate(function) {
            self.finish(function, false, false);
            return false;
        }
        true
    }
    fn should_verify(&self) -> bool {
        if self.run.change {
            return true;
        }
        if self.iteration > 1 {
            return self.history[self.iteration as usize - 1]
                < 1e-4 * self.history[self.iteration as usize - 2];
        }
        self.run.objective < 1e-6 * self.history[0]
    }
    fn step_is_small<F: FunctionSetWithDerivatives>(
        &mut self,
        function: &mut F,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> bool {
        for index in 1..=self.delta.length() {
            self.delta[index] = self.previous[index] - self.sol[index];
        }
        criterion(self, function)
    }
    fn analyze_progress<F: FunctionSetWithDerivatives>(
        &mut self,
        function: &mut F,
        verified: bool,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> bool {
        if self.iteration > 5
            && self.run.objective >= 0.95 * self.history[self.iteration as usize - 5]
        {
            if self.run.change {
                self.finish(function, true, false);
                return false;
            }
            self.run.change = true;
        } else {
            self.run.change = false;
        }
        if self.run.gradient_norm > 0.9 * self.run.old_gradient
            && self.run.objective > 0.5 * self.run.previous_minimum
        {
            self.run.change = true;
        }
        if !self.run.change && !verified && self.step_is_small(function, criterion) {
            self.finish(function, true, true);
            return false;
        }
        true
    }
}
