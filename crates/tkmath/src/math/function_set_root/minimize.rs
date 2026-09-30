use super::{FunctionSetRoot, FunctionSetWithDerivatives, Vector, line::Line};
use crate::math::{BrentMinimum, Function};
impl FunctionSetRoot {
    pub(super) fn minimize_three(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
    ) -> bool {
        let mut tolerance: f64 = direction_tolerance(&self.delta, &self.tol, 2.1, 1e-16);
        if tolerance > 1.9 {
            return false;
        }
        tolerance /= 3.0;
        self.delta
            .initialized(&self.saved.subtracted(&self.previous).unwrap())
            .unwrap();
        let norm: f64 = self.delta.norm_unchecked();
        if norm <= 1e-16 {
            return false;
        }
        let reciprocal: f64 = 1.0 / norm;
        let last: f64 = self.sol.subtracted(&self.saved).unwrap().norm_unchecked() * reciprocal;
        if last < 1e-2 {
            return false;
        }
        let mut line: Line<'_, _> = Line::new(
            function,
            &self.saved,
            &self.delta,
            &mut self.line_point,
            &mut self.line_residual,
        );
        let mut solver: BrentMinimum = BrentMinimum::with_options(tolerance, 100, tolerance);
        solver.perform(&mut line, -1.0, 0.0, last);
        if !solver.is_done()
            || solver
                .minimum()
                .unwrap()
                .partial_cmp(&self.run.old_objective)
                != Some(std::cmp::Ordering::Less)
        {
            return false;
        }
        self.delta.multiply(solver.location().unwrap());
        true
    }
    pub(super) fn minimize_two(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        let value: f64 = self.run.old_objective;
        let next: f64 = self.run.objective;
        if value.abs() >= 1e100 || next.abs() >= 1e100 {
            return false;
        }
        let tolerance: f64 = direction_tolerance(&self.delta, &self.tol, 1.1, 1e-20);
        if tolerance > 0.9 {
            return false;
        }
        let df1: f64 = self.saved_gradient.dot_unchecked(&self.delta);
        let df2: f64 = self.gradient.dot_unchecked(&self.delta);
        let mut parameter: f64 = interpolate(value, next, df1, df2);
        if parameter.abs() >= 1.0 {
            return false;
        }
        let mut line: Line<'_, _> = Line::new(
            function,
            &self.saved,
            &self.delta,
            &mut self.line_point,
            &mut self.line_residual,
        );
        let mut estimate: f64 = 0.0;
        line.value(parameter, &mut estimate);
        let mut good: bool = estimate < value;
        let mut best: f64 = if good { estimate } else { value };
        if estimate > 0.2 * value && tolerance < 0.5 {
            improve(
                &mut line,
                tolerance,
                &mut parameter,
                &mut best,
                &mut good,
                self.saved_gradient.norm2_unchecked(),
            );
        }
        if good {
            self.delta.multiply(parameter);
        }
        good
    }
}
fn direction_tolerance(
    direction: &Vector<'_>,
    tolerance: &Vector<'_>,
    mut result: f64,
    epsilon: f64,
) -> f64 {
    for index in 1..=tolerance.length() {
        let magnitude: f64 = direction[index].abs();
        if magnitude > epsilon && tolerance[index] / magnitude < result {
            result = tolerance[index] / magnitude;
        }
    }
    result
}
fn interpolate(value: f64, next: f64, first_derivative: f64, last_derivative: f64) -> f64 {
    if first_derivative < -1e-20 && last_derivative > 1e-20 {
        return -first_derivative / (last_derivative - first_derivative);
    }
    let a: f64 = next - (first_derivative + value);
    let b: f64 = first_derivative;
    if a.abs() <= 1e-20 {
        return if b.abs() >= 1e-20 { -value / b } else { 0.0 };
    }
    let delta: f64 = b * b - 4.0 * a * value;
    if delta.partial_cmp(&1e-9) != Some(std::cmp::Ordering::Greater) {
        return -(0.5 * b) / a;
    }
    let root: f64 = delta.sqrt();
    let mut parameter: f64 = -(b + root);
    let alternative: f64 = root - b;
    if alternative.abs() < parameter.abs() {
        parameter = alternative;
    }
    parameter / (2.0 * a)
}
fn improve(
    line: &mut impl Function,
    tolerance: f64,
    parameter: &mut f64,
    best: &mut f64,
    good: &mut bool,
    gradient_norm: f64,
) {
    let first: f64 = if *parameter < 0.0 { *parameter } else { 0.0 };
    let middle: f64 = if *parameter < 0.0 { 0.0 } else { *parameter };
    let mut solver: BrentMinimum = BrentMinimum::with_options(tolerance, 100, tolerance);
    solver.perform(line, first, middle, 1.0);
    if !accept(&solver, parameter, best, good) {
        return;
    }
    if gradient_norm > 1.0 / (1e-7 * 1e-7) && *parameter > first && *parameter < 1.0 {
        solver.perform(line, first, (first + *parameter) / 2.0, *parameter);
        accept(&solver, parameter, best, good);
        solver.perform(line, *parameter, (1.0 + *parameter) / 2.0, 1.0);
        accept(&solver, parameter, best, good);
    }
}
fn accept(solver: &BrentMinimum, parameter: &mut f64, best: &mut f64, good: &mut bool) -> bool {
    if !solver.is_done()
        || !matches!(
            solver.minimum().unwrap().partial_cmp(best),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
    {
        return false;
    }
    *parameter = solver.location().unwrap();
    *best = solver.minimum().unwrap();
    *good = true;
    true
}
