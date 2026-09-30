use super::super::{BracketMinimum, BrentMinimum, Function, optimization::line::ObjectiveLine};
use super::bounds::{initial_scale, limit_scales, native_max, native_min, tangent_direction};
use super::{Bfgs, MultipleVarFunctionWithGradient, Vector};
impl Bfgs {
    fn scale_interval(&self, direction: &mut Vector<'static>) -> Option<(f64, [f64; 2])> {
        let mut scale: f64 = initial_scale(self.result.minimum, direction, &self.result.gradient)?;
        let mut limits: [f64; 2] = [-2e100, 2e100];
        let Some((left, right)) = &self.bounds else {
            return Some((scale, limits));
        };
        if !limit_scales(&self.result.location, direction, left, right, &mut limits) {
            return None;
        }
        if limits[0] > -1e-9 && limits[1] < 1e-9 {
            tangent_direction(&self.result.location, direction, left, right);
            scale = initial_scale(self.result.minimum, direction, &self.result.gradient)?;
            if !limit_scales(&self.result.location, direction, left, right, &mut limits) {
                return None;
            }
        }
        scale = native_min(scale, limits[1]);
        scale = native_max(scale, limits[0]);
        Some((scale, limits))
    }
    pub(super) fn minimize_direction(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
        direction: &mut Vector<'static>,
    ) -> bool {
        let Some((scale, limits)) = self.scale_interval(direction) else {
            return false;
        };
        let mut line: ObjectiveLine<'_, _> = ObjectiveLine {
            function,
            origin: &self.result.location,
            direction,
            point: Vector::with_bounds(1, direction.upper()).unwrap(),
        };
        let Some((step, value)) = search(
            &mut line,
            scale,
            limits,
            self.result.minimum,
            self.bounds.is_some(),
        ) else {
            return false;
        };
        self.result.minimum = value;
        direction.multiply(step);
        self.result.location.add(direction).unwrap();
        true
    }
}
fn search(
    function: &mut impl Function,
    scale: f64,
    limits: [f64; 2],
    initial: f64,
    bounded: bool,
) -> Option<(f64, f64)> {
    let mut value: f64 = 0.0;
    if !function.value(scale, &mut value) {
        return None;
    }
    let mut bracket: BracketMinimum = BracketMinimum::new(0.0, scale);
    if bounded {
        bracket.set_limits(limits[0], limits[1]);
    }
    bracket.set_first_value(initial);
    bracket.set_second_value(value);
    bracket.perform(function);
    if bracket.is_done() {
        return bracket_search(function, &bracket);
    }
    if bounded {
        return boundary_search(function, limits);
    }
    None
}
fn bracket_search(function: &mut impl Function, bracket: &BracketMinimum) -> Option<(f64, f64)> {
    let [first, middle, last]: [f64; 3] = bracket.values().unwrap();
    let mut solver: BrentMinimum =
        BrentMinimum::with_value(1e-3, bracket.function_values().unwrap()[1], 100, 1e-8);
    solver.perform(function, first, middle, last);
    if !solver.is_done() {
        return None;
    }
    Some((solver.location().unwrap(), solver.minimum().unwrap()))
}
fn boundary_search(function: &mut impl Function, limits: [f64; 2]) -> Option<(f64, f64)> {
    let (mut low, mut high): (f64, f64) = (0.0, 0.0);
    if !function.value(limits[0], &mut low) || !function.value(limits[1], &mut high) {
        return None;
    }
    if low < high {
        Some((limits[0], low))
    } else {
        Some((limits[1], high))
    }
}
