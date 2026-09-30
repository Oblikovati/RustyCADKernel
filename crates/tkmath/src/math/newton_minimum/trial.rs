use super::{MultipleVarFunctionWithHessian, NewtonMinimum, Status, Vector};
impl NewtonMinimum {
    pub(super) fn evaluate_trial(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        previous: &Vector<'static>,
        next: &mut Vector<'static>,
    ) {
        loop {
            next.set_difference(previous, &self.step).unwrap();
            if function.value(next, &mut self.result.minimum) {
                return;
            }
            self.step.divide(2.0).unwrap();
        }
    }
    pub(super) fn correct_trial(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        previous: &mut Vector<'static>,
        next: &mut Vector<'static>,
        old_value: f64,
        consecutive: i32,
    ) -> bool {
        let mut value: f64 = self.result.minimum;
        self.result.minimum = self.result.previous;
        self.reduce_trial(function, previous, next, old_value, &mut value);
        if value
            .partial_cmp(&old_value)
            .is_none_or(|order| order == std::cmp::Ordering::Greater)
        {
            self.result.status = Status::DirectionSearchError;
            return false;
        }
        std::mem::swap(previous, next);
        self.result.previous = old_value;
        self.result.minimum = value;
        let good: bool = self.result.iterations < self.result.limit;
        if !good && consecutive < 2 {
            self.result.status = Status::TooManyIterations;
        }
        good
    }
    fn reduce_trial(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        previous: &Vector<'static>,
        next: &mut Vector<'static>,
        old_value: f64,
        value: &mut f64,
    ) {
        let mut reductions: i32 = 0;
        while *value > old_value && reductions < 10 {
            self.step.multiply(0.4);
            next.set_difference(previous, &self.step).unwrap();
            function.value(next, value);
            reductions += 1;
        }
    }
}
