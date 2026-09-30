use super::{Failure, MultipleVarFunctionWithHessian, NewtonMinimum, Status, Vector};
impl NewtonMinimum {
    pub(super) fn iterate(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        [previous, next]: [&mut Vector<'static>; 2],
        criterion: &mut impl FnMut(&Self) -> bool,
    ) -> Result<(), Failure> {
        let mut consecutive: i32 = 0;
        let mut value: f64 = 0.0;
        loop {
            self.result.iterations += 1;
            if !self.prepare_step(function, previous, &mut value)? {
                return Ok(());
            }
            self.project_step(previous, next);
            self.evaluate_trial(function, previous, next);
            consecutive = if criterion(self) { consecutive + 1 } else { 0 };
            let good: bool = self.correct_trial(function, previous, next, value, consecutive);
            if !good || consecutive >= 2 {
                break;
            }
        }
        self.result.location.initialized(previous)
    }
    fn prepare_step(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        point: &Vector<'_>,
        value: &mut f64,
    ) -> Result<bool, Failure> {
        if !function.values_hessian(point, value, &mut self.result.gradient, &mut self.hessian) {
            self.fail(Status::FunctionError);
            return Ok(false);
        }
        if self.result.iterations == 1 {
            self.result.previous = *value;
            self.result.minimum = *value;
        }
        Ok(self.adjust_curvature()? && self.solve_step()?)
    }
}
