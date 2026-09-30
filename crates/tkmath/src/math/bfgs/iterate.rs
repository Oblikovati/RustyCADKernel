use super::{Bfgs, Matrix, MultipleVarFunctionWithGradient, Status, Vector};
impl Bfgs {
    pub(super) fn iterate<F: MultipleVarFunctionWithGradient>(
        &mut self,
        function: &mut F,
        direction: &mut Vector<'static>,
        hessian: &mut Matrix<'static>,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) {
        self.result.iterations = 1;
        while self.result.iterations <= self.result.limit {
            if self.iteration(function, direction, hessian, criterion) {
                return;
            }
            self.result.iterations += 1;
        }
        self.result.fail(Status::TooManyIterations);
    }
    fn iteration<F: MultipleVarFunctionWithGradient>(
        &mut self,
        function: &mut F,
        direction: &mut Vector<'static>,
        hessian: &mut Matrix<'static>,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> bool {
        self.result.minimum = self.result.previous;
        let good: bool = self.minimize_direction(function, direction);
        // OCC30492: even a failed direction can satisfy the objective stopping test.
        if criterion(self, function) {
            self.result.succeed();
            return true;
        }
        if !good {
            self.result.fail(Status::DirectionSearchError);
            return true;
        }
        self.result.previous = self.result.minimum;
        let mut difference: Vector<'static> = self.result.gradient.clone();
        if !function.values(
            &self.result.location,
            &mut self.result.minimum,
            &mut self.result.gradient,
        ) {
            self.result.fail(Status::FunctionError);
            return true;
        }
        for index in 1..=difference.upper() {
            difference[index] = self.result.gradient[index] - difference[index];
        }
        super::update::update_hessian(hessian, direction, &mut difference);
        super::update::next_direction(hessian, &self.result.gradient, direction);
        false
    }
}
