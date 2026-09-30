//! MathRoot_Secant.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::bracket::finite_value;
use super::utils::is_valid_config;
use crate::math::Function;
use crate::math_utils::{Config, ScalarResult, Status, is_x_converged};
fn finish(mut result: ScalarResult, status: Status, x: f64, value: f64) -> ScalarResult {
    result.status = status;
    result.root = Some(x);
    result.value = Some(value);
    result
}
/// Uses the native secant step and residual check after small steps; `secant(&mut f,1.0,2.0,&Config::default())`.
pub fn secant(f: &mut impl Function, mut x0: f64, mut x1: f64, config: &Config) -> ScalarResult {
    let mut result: ScalarResult = ScalarResult::default();
    if !x0.is_finite() || !x1.is_finite() || x0 == x1 || !is_valid_config(config) {
        result.status = Status::InvalidInput;
        return result;
    }
    let (mut f0, mut f1): (f64, f64) = (0.0, 0.0);
    if !finite_value(f, x0, &mut f0, &mut result) || !finite_value(f, x1, &mut f1, &mut result) {
        return result;
    }
    if f0.abs() <= config.f_tolerance {
        return finish(result, Status::Ok, x0, f0);
    }
    for iteration in 0..config.max_iterations {
        result.iterations = iteration + 1;
        if f1.abs() <= config.f_tolerance {
            return finish(result, Status::Ok, x1, f1);
        }
        let denominator: f64 = f1 - f0;
        if !denominator.is_finite() || denominator == 0.0 {
            return finish(result, Status::NumericalError, x1, f1);
        }
        let next: f64 = x1 - (f1 / denominator) * (x1 - x0);
        if !next.is_finite() {
            return finish(result, Status::NumericalError, x1, f1);
        }
        if is_x_converged(x1, next, config.x_tolerance) {
            return converged_step(f, result, [x1, f1, next], config);
        }
        x0 = x1;
        f0 = f1;
        x1 = next;
        if !finite_value(f, x1, &mut f1, &mut result) {
            let status: Status = result.status;
            return finish(result, status, x0, f0);
        }
    }
    finish(result, Status::MaxIterations, x1, f1)
}
fn converged_step(
    f: &mut impl Function,
    mut result: ScalarResult,
    points: [f64; 3],
    config: &Config,
) -> ScalarResult {
    let [previous, previous_value, next]: [f64; 3] = points;
    let mut value: f64 = 0.0;
    if !finite_value(f, next, &mut value, &mut result) {
        let status: Status = result.status;
        return finish(result, status, previous, previous_value);
    }
    let status: Status = if value.abs() <= config.f_tolerance {
        Status::Ok
    } else {
        Status::NotConverged
    };
    finish(result, status, next, value)
}
/// Constructs the native one-percent initial perturbation; `secant_auto(&mut f,1.0,&Config::default())`.
pub fn secant_auto(f: &mut impl Function, x: f64, config: &Config) -> ScalarResult {
    let delta: f64 = if x.abs() > 1.0 { 0.01 * x } else { 0.01 };
    if !x.is_finite() || !(x + delta).is_finite() || x + delta == x {
        return ScalarResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    secant(f, x, x + delta, config)
}
