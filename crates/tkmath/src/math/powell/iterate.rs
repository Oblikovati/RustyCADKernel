use super::{MultipleVarFunction, Powell, Status, Vector};
impl Powell {
    pub(super) fn iteration<F: MultipleVarFunction>(
        &mut self,
        function: &mut F,
        [previous, direction]: [&mut Vector<'static>; 2],
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> bool {
        function.value(&self.result.location, &mut self.result.previous);
        let Some((largest, column)) = self.search_columns(function, direction) else {
            self.result.fail(Status::DirectionSearchError);
            return true;
        };
        if criterion(self, function) {
            self.state = function.state_number();
            self.result.succeed();
            return true;
        }
        if self.result.iterations == self.result.limit {
            self.result.fail(Status::TooManyIterations);
            return true;
        }
        if self.extrapolate(function, previous, direction, largest, column) {
            return false;
        }
        // A failed extrapolated line search is reported as math_FunctionError natively.
        self.result.fail(Status::FunctionError);
        true
    }
    fn search_columns(
        &mut self,
        function: &mut impl MultipleVarFunction,
        direction: &mut Vector<'static>,
    ) -> Option<(f64, i32)> {
        let (mut largest, mut column): (f64, i32) = (0.0, 0);
        for index in 1..=direction.upper() {
            for row in 1..=direction.upper() {
                direction[row] = self.directions[(row, index)];
            }
            function.value(&self.result.location, &mut self.scratch_value);
            if !self.minimize_direction(function, direction) {
                return None;
            }
            if (self.scratch_value - self.result.minimum).abs() > largest {
                largest = (self.scratch_value - self.result.minimum).abs();
                column = index;
            }
        }
        Some((largest, column))
    }
}
