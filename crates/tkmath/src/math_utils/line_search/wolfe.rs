use super::*;
struct WolfeState<'a, 'b> {
    point: &'a Vector<'b>,
    direction: &'a Vector<'b>,
    candidate: Vector<'static>,
    gradient: Vector<'static>,
    fx: f64,
    derivative: f64,
    c1: f64,
    c2: f64,
    result: LineSearchResult,
}
impl WolfeState<'_, '_> {
    fn value(&mut self, f: &mut impl MultipleVarFunction, alpha: f64) -> Option<f64> {
        if !new_point(self.point, self.direction, alpha, &mut self.candidate) {
            self.result.status = Status::NumericalError;
            return None;
        }
        let value: f64 = self.result.evaluate(f, &self.candidate)?;
        self.result.alpha = alpha;
        self.result.f_new = value;
        self.result.finite(value).then_some(value)
    }
    fn slope(&mut self, f: &mut impl MultipleVarFunctionWithGradient) -> Option<f64> {
        self.result.evaluations += 1;
        if !f.gradient(&self.candidate, &mut self.gradient) {
            self.result.status = Status::CallbackError;
            return None;
        }
        let mut derivative: f64 = 0.0;
        for index in 0..self.point.length() as usize {
            let value: f64 = *self.gradient.at(index).unwrap();
            if !self.result.finite(value) {
                return None;
            }
            derivative += value * self.direction.at(index).unwrap();
        }
        self.result.finite(derivative).then_some(derivative)
    }
    fn curvature(&mut self, slope: f64) -> bool {
        if slope.abs() <= -self.c2 * self.derivative {
            self.result.success();
            return true;
        }
        false
    }
    fn insufficient(&self, alpha: f64, value: f64) -> bool {
        value > self.fx + self.c1 * alpha * self.derivative
    }
    fn bracket(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        mut alpha: f64,
        iterations: u32,
    ) -> Option<[f64; 3]> {
        let mut previous: f64 = 0.0;
        let mut previous_value: f64 = self.fx;
        for iteration in 0..iterations {
            let value: f64 = self.value(f, alpha)?;
            if self.insufficient(alpha, value) || (iteration > 0 && value >= previous_value) {
                return Some([previous, alpha, previous_value]);
            }
            let slope: f64 = self.slope(f)?;
            if self.curvature(slope) {
                return None;
            }
            if slope >= 0.0 {
                return Some([alpha, previous, value]);
            }
            previous = alpha;
            previous_value = value;
            alpha *= 2.0;
            if !self.result.finite(alpha) {
                return None;
            }
        }
        self.result.status = Status::MaxIterations;
        None
    }
    fn zoom(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        bracket: [f64; 3],
        iterations: u32,
    ) {
        let [mut low, mut high, mut phi_low]: [f64; 3] = bracket;
        for _ in 0..iterations {
            let alpha: f64 = 0.5 * low + 0.5 * high;
            if alpha == low || alpha == high {
                self.result.status = Status::NotConverged;
                return;
            }
            let Some(value) = self.value(f, alpha) else {
                return;
            };
            if self.insufficient(alpha, value) || value >= phi_low {
                high = alpha;
                continue;
            }
            let Some(slope) = self.slope(f) else {
                return;
            };
            if self.curvature(slope) {
                return;
            }
            if slope * (high - low) >= 0.0 {
                high = low;
            }
            low = alpha;
            phi_low = value;
        }
        self.result.status = Status::MaxIterations;
    }
}
/// Uses strong Wolfe conditions with [initial alpha,c1,c2]; `wolfe_search(&mut f,&x,&d,&g,fx,[1.0,1e-4,0.9],20)`.
pub fn wolfe_search(
    f: &mut impl MultipleVarFunctionWithGradient,
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
    fx: f64,
    parameters: [f64; 3],
    iterations: u32,
) -> LineSearchResult {
    let [alpha, c1, c2]: [f64; 3] = parameters;
    let mut result: LineSearchResult = LineSearchResult::default();
    if !common_valid(point, direction, gradient, fx, alpha, iterations)
        || !c1.is_finite()
        || !c2.is_finite()
        || c1 <= 0.0
        || c1 >= c2
        || c2 >= 1.0
    {
        result.status = Status::InvalidInput;
        return result;
    }
    let derivative: f64 = match directional(point, direction, gradient) {
        Ok(value) => value,
        Err(status) => {
            result.status = status;
            return result;
        }
    };
    let mut state: WolfeState<'_, '_> = WolfeState {
        point,
        direction,
        fx,
        derivative,
        c1,
        c2,
        result,
        candidate: Vector::with_size(point.length() as usize),
        gradient: Vector::with_size(gradient.length() as usize),
    };
    if let Some(bracket) = state.bracket(f, alpha, iterations) {
        state.zoom(f, bracket, iterations);
    }
    state.result
}
