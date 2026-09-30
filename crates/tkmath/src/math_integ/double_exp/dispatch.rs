//! MathInteg_DoubleExp.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{DoubleExpConfig, exp_sinh, sinh_sinh, tanh_sinh};
use crate::math_integ::kronrod::{invalid, reversed};
use crate::{
    math::Function,
    math_utils::{IntegResult, Status},
};
/// Selects the native transform from interval bounds; `double_exponential(&mut f, 0.0, f64::INFINITY, &DoubleExpConfig::default())`.
pub fn double_exponential(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &DoubleExpConfig,
) -> IntegResult {
    if lower.is_nan()
        || upper.is_nan()
        || (lower > upper && (!lower.is_finite() || !upper.is_finite()))
    {
        return invalid();
    }
    if lower == f64::INFINITY || upper == f64::NEG_INFINITY {
        return invalid();
    }
    if lower == f64::NEG_INFINITY && upper == f64::INFINITY {
        return sinh_sinh(f, config);
    }
    if upper == f64::INFINITY {
        return exp_sinh(f, lower, config);
    }
    if lower == f64::NEG_INFINITY {
        let mut reflected = |x: f64, value: &mut f64| f.value(-x, value);
        return exp_sinh(&mut reflected, -upper, config);
    }
    tanh_sinh(f, lower, upper, config)
}
/// Native singular endpoint convenience configuration; `tanh_sinh_singular(&mut f, 0.0, 1.0, 1e-8)`.
pub fn tanh_sinh_singular(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    tolerance: f64,
) -> IntegResult {
    let config: DoubleExpConfig = DoubleExpConfig::new(tolerance, 100);
    tanh_sinh(f, lower, upper, &config)
}
/// Splits at a supplied interior singularity; `tanh_sinh_with_singularity(&mut f, 0.0, 1.0, 0.5, &DoubleExpConfig::default())`.
pub fn tanh_sinh_with_singularity(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    singularity: f64,
    config: &DoubleExpConfig,
) -> IntegResult {
    if !singularity.is_finite() {
        return invalid();
    }
    if lower.is_finite() && upper.is_finite() && lower > upper {
        return reversed(tanh_sinh_with_singularity(
            f,
            upper,
            lower,
            singularity,
            config,
        ));
    }
    if singularity <= lower || singularity >= upper {
        return tanh_sinh(f, lower, upper, config);
    }
    let left: IntegResult = tanh_sinh(f, lower, singularity, config);
    if !left.is_done() {
        return left;
    }
    let right: IntegResult = tanh_sinh(f, singularity, upper, config);
    combine_singular_parts(left, right)
}
fn combine_singular_parts(left: IntegResult, mut right: IntegResult) -> IntegResult {
    if !right.is_done() {
        if let (Some(a), Some(b)) = (left.value, right.value) {
            right.value = Some(a + b);
        }
        right.number_of_points += left.number_of_points;
        right.iterations += left.iterations;
        return right;
    }
    let value: f64 = left.value.unwrap() + right.value.unwrap();
    let error: Option<f64> = left
        .absolute_error
        .zip(right.absolute_error)
        .map(|(a, b)| a + b);
    IntegResult {
        status: Status::Ok,
        value: Some(value),
        absolute_error: error,
        relative_error: error
            .filter(|_| value.abs() > f64::EPSILON)
            .map(|error| error / value.abs()),
        number_of_points: left.number_of_points + right.number_of_points,
        iterations: left.iterations + right.iterations,
    }
}
