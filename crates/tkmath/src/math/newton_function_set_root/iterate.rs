use super::{Failure, FunctionSetWithDerivatives, NewtonFunctionSetRoot, Vector};
use crate::math::recipes::{STATUS_OK, lu_decompose_with_scales, lu_solve};
impl NewtonFunctionSetRoot {
    pub(super) fn iterate<F: FunctionSetWithDerivatives>(
        &mut self,
        function: &mut F,
        [lower, upper]: [&Vector<'_>; 2],
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> Result<(), Failure> {
        for iteration in 1..=self.limit {
            self.iteration = iteration;
            if !self.newton_step(lower, upper)? {
                return Ok(());
            }
            if !function.values(&self.solution, &mut self.residual, &mut self.jacobian) {
                return Ok(());
            }
            if criterion(self, function) {
                self.state = function.state_number();
                self.done = true;
                return Ok(());
            }
        }
        self.iteration = self.limit.saturating_add(1).max(1);
        Ok(())
    }
    fn newton_step(&mut self, lower: &Vector<'_>, upper: &Vector<'_>) -> Result<bool, Failure> {
        for index in 1..=self.delta.length() {
            self.delta[index] = -self.residual[index];
        }
        // LU_Decompose factors the Jacobian in place, leaving a partial factorization on
        // failure; the next F.Values call overwrites it.
        let mut sign = 0.0;
        let status = lu_decompose_with_scales(
            &mut self.jacobian,
            &mut self.pivots,
            &mut sign,
            &mut self.scratch,
            1.0e-30,
        )?;
        if status != STATUS_OK {
            return Ok(false);
        }
        lu_solve(&self.jacobian, &self.pivots, &mut self.delta)?;
        self.clip_trial(lower, upper)?;
        Ok(true)
    }
    fn clip_trial(&mut self, lower: &Vector<'_>, upper: &Vector<'_>) -> Result<(), Failure> {
        for index in 1..=self.solution.length() {
            self.solution[index] += self.delta[index];
            let low: f64 = *lower.value(index)?;
            let high: f64 = *upper.value(index)?;
            if self.solution[index] <= low {
                self.solution[index] = low;
            }
            if self.solution[index] >= high {
                self.solution[index] = high;
            }
        }
        Ok(())
    }
}
