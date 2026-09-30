use super::ZERO;
fn evaluate(coefficients: &[f64], x: f64) -> f64 {
    let mut value: f64 = coefficients[0];
    for coefficient in &coefficients[1..] {
        value = value * x + coefficient;
    }
    value
}
fn values(coefficients: &[f64], x: f64) -> (f64, f64) {
    let mut value: f64 = coefficients[0] * x + coefficients[1];
    let mut derivative: f64 = coefficients[0];
    for coefficient in &coefficients[2..] {
        derivative = derivative * x + value;
        value = value * x + coefficient;
    }
    (value, derivative)
}
pub(super) fn refine(coefficients: &[f64], guess: f64) -> f64 {
    let mut solution: f64 = guess;
    let initial: f64 = evaluate(coefficients, guess);
    let mut value: f64 = 0.0;
    for _ in 1..10 {
        let derivative: f64;
        (value, derivative) = values(coefficients, solution);
        if derivative.abs() <= ZERO {
            break;
        }
        let delta: f64 = -value / derivative;
        if delta.abs() <= f64::EPSILON * solution.abs() {
            break;
        }
        solution += delta;
    }
    // Native compares the last pre-update residual, not a fresh evaluation at solution.
    if value.abs() <= initial.abs() {
        solution
    } else {
        guess
    }
}
