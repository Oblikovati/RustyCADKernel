use super::{FunctionSetRoot, FunctionSetWithDerivatives, direction};
impl FunctionSetRoot {
    pub(super) fn boundary_descent(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
    ) -> bool {
        self.run.stop = self.run.objective > 1.001 * self.run.old_objective;
        self.run.boundary = self.run.outside;
        self.run.descent = 0;
        while self.run.boundary
            && (self.run.objective < self.run.old_objective || self.run.descent == 0)
            && !self.run.stop
        {
            if !self.boundary_trial(function) {
                return false;
            }
            if !self.conditional_descent(function) {
                return false;
            }
            self.run.stop = self.run.slope >= 0.0 || self.run.descent >= 10;
        }
        let repair: bool = (self.run.objective / self.run.previous_minimum > 0.005
            && self.run.objective >= self.run.old_objective)
            || self.run.objective >= self.run.previous_minimum;
        if repair && !self.repair_boundary(function) {
            return false;
        }
        true
    }
    fn boundary_trial(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        self.run.descent += 1;
        self.saved.initialized(&self.sol).unwrap();
        self.run.old_objective = self.run.objective;
        self.run.slope = direction::constrained(
            &self.jacobian,
            &self.gradient,
            &self.residual,
            &self.constraints,
            self.run.change,
            &self.inverse_lengths,
            &mut self.direction,
        );
        if self.run.slope.partial_cmp(&-1e-32) != Some(std::cmp::Ordering::Less) {
            self.run.stop = true;
            return true;
        }
        self.choose_lambda();
        self.advance();
        if self.check_divergence() {
            return false;
        }
        self.run.boundary = self.trim_bounds();
        self.saved_gradient.initialized(&self.gradient).unwrap();
        if !self.evaluate_or_finish(function) {
            return false;
        }
        self.run.lambda2 = self.run.gradient_norm;
        true
    }
    fn conditional_descent(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        while self.run.objective / self.run.previous_minimum > 0.005
            && self.run.objective < self.run.old_objective
            && !self.run.stop
        {
            self.run.descent += 1;
            if self.run.objective < self.run.old_objective
                && self.run.slope < 0.0
                && !self.conditional_trial()
            {
                return false;
            }
            self.saved_gradient.initialized(&self.gradient).unwrap();
            if !self.evaluate_or_finish(function) {
                return false;
            }
            self.run.lambda2 = self.run.gradient_norm;
            self.run.slope = self.gradient.dot_unchecked(&self.direction);
            self.run.stop = self.run.slope >= 0.0 || self.run.descent >= 10 || self.run.boundary;
        }
        true
    }
    fn conditional_trial(&mut self) -> bool {
        self.saved.initialized(&self.sol).unwrap();
        self.run.old_objective = self.run.objective;
        self.advance();
        if self.check_divergence() {
            return false;
        }
        self.run.boundary = self.trim_bounds();
        true
    }
    fn repair_boundary(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        self.run.descent += 1;
        self.run.good = self.minimize_two(function);
        if !self.run.good {
            self.sol.initialized(&self.saved).unwrap();
            self.run.outside = false;
        } else {
            self.sol
                .initialized(&self.saved.added(&self.delta).unwrap())
                .unwrap();
            if self.check_divergence() {
                return false;
            }
            self.run.outside = self.trim_bounds();
            if !self.evaluate_or_finish(function) {
                return false;
            }
        }
        self.run.slope = self.gradient.dot_unchecked(&self.direction);
        true
    }
}
