use super::{NewtonMinimum, Vector};
impl NewtonMinimum {
    pub(super) fn project_step(&mut self, previous: &Vector<'static>, next: &mut Vector<'static>) {
        let Some((left, right)) = &self.bounds else {
            return;
        };
        next.set_difference(previous, &self.step).unwrap();
        let multiplier: f64 = step_multiplier(previous, next, &self.step, left, right);
        if multiplier == f64::MAX {
            return;
        }
        if multiplier > 1e-9 {
            self.step.multiply(multiplier);
            return;
        }
        zero_outward(previous, &mut self.step, left, right);
    }
}
fn step_multiplier(
    previous: &Vector<'_>,
    next: &Vector<'_>,
    step: &Vector<'_>,
    left: &Vector<'_>,
    right: &Vector<'_>,
) -> f64 {
    let mut multiplier: f64 = f64::MAX;
    for index in 1..=left.upper() {
        let magnitude: f64 = step[index].abs();
        if magnitude < f64::MIN_POSITIVE {
            continue;
        }
        if next[index] < left[index] {
            multiplier = native_min(
                (previous[index] - left[index]).abs() / magnitude,
                multiplier,
            );
        }
        if next[index] > right[index] {
            multiplier = native_min(
                (previous[index] - right[index]).abs() / magnitude,
                multiplier,
            );
        }
    }
    multiplier
}
fn native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}
fn zero_outward(
    previous: &Vector<'_>,
    step: &mut Vector<'_>,
    left: &Vector<'_>,
    right: &Vector<'_>,
) {
    for index in 1..=left.upper() {
        if ((previous[index] - right[index]).abs() < 1e-9 && step[index] < 0.0)
            || ((previous[index] - left[index]).abs() < 1e-9 && step[index] > 0.0)
        {
            step[index] = 0.0;
        }
    }
}
