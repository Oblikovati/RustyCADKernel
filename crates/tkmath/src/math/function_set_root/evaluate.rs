use super::{FunctionSetRoot, FunctionSetWithDerivatives};
impl FunctionSetRoot {
    pub(super) fn evaluate(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        if !function.values(&self.sol, &mut self.residual, &mut self.jacobian) {
            return false;
        }
        for index in 1..=self.residual.length() {
            if self.residual[index] <= -1e100 || self.residual[index] >= 1e100 {
                return false;
            }
        }
        self.run.objective = 0.5 * self.residual.norm2_unchecked();
        self.gradient
            .set_transpose_matrix_vector(&self.jacobian, &self.residual)
            .unwrap();
        for index in 1..=self.gradient.length() {
            if self.gradient[index].abs() >= 1e100 {
                return false;
            }
        }
        self.run.gradient_norm = self.gradient.norm2_unchecked();
        true
    }
    pub(super) fn evaluate_or_finish(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
    ) -> bool {
        if !self.run.new_solution {
            return true;
        }
        if self.evaluate(function) {
            return true;
        }
        self.finish(function, false, false);
        false
    }
}
