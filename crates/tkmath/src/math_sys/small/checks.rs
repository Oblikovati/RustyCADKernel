//! `MathSys::Utils` helpers of MathSys_NewtonTypes.hxx and the Newton2D gradient floors
//! at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math_poly::LongDouble as E;
use rustycad_tkernel::precision;

/// Armijo backtracking halvings per Newton iteration; `LINE_SEARCH_MAX == 8`.
pub const LINE_SEARCH_MAX: usize = 8;
/// Sufficient-decrease constant of the scaled Armijo test; `ARMIJO_C1 == 1e-4`.
pub const ARMIJO_C1: f64 = 1.0e-4;
/// Relative pivot floor of the fixed-size linear solves; `NEWTON_PIVOT_TOL == 64.0 * f64::EPSILON`.
pub const NEWTON_PIVOT_TOL: f64 = 64.0 * precision::computational();
/// Gradient norm below which the steepest-descent fallback reports `Singular`; `CRITICAL_GRAD == 1e-30`.
pub const CRITICAL_GRAD: f64 = 1.0e-30;
/// Squared companion of [`CRITICAL_GRAD`], declared but unused upstream; `CRITICAL_GRAD_SQ == 1e-60`.
pub const CRITICAL_GRAD_SQ: f64 = 1.0e-60;

/// True when every entry is finite; `is_finite_array(&[1.0, 2.0])`.
pub fn is_finite_array<const N: usize>(values: &[f64; N]) -> bool {
    values.iter().all(|value| value.is_finite())
}

/// Sequential `hypot` norm that avoids overflow and underflow; `safe_norm_n(&[3.0, 4.0]) == 5.0`.
pub fn safe_norm_n<const N: usize>(values: &[f64; N]) -> f64 {
    values
        .iter()
        .fold(0.0_f64, |norm, value| norm.hypot(*value))
}

/// Sequential `hypot` distance `|left - right|`; `safe_distance_n(&[3.0, 4.0], &[0.0; 2]) == 5.0`.
pub fn safe_distance_n<const N: usize>(left: &[f64; N], right: &[f64; N]) -> f64 {
    (0..N).fold(0.0_f64, |norm, i| norm.hypot(left[i] - right[i]))
}

/// True unless some `|left - right|` exceeds the tolerance (a NaN difference passes, as
/// upstream's `>` test does); `is_step_within_tolerance(&[1.0], &[1.5], 1.0)`.
pub fn is_step_within_tolerance<const N: usize>(
    left: &[f64; N],
    right: &[f64; N],
    tolerance: f64,
) -> bool {
    (0..N).all(|i| {
        (left[i] - right[i]).abs().partial_cmp(&tolerance) != Some(std::cmp::Ordering::Greater)
    })
}

/// Directional derivative `F . (J step)` of the merit `|F|^2 / 2`, accumulated in x87 long
/// double row by row as upstream does; `merit_directional_derivative(&f, &j, &step)`.
pub fn merit_directional_derivative<const N: usize>(
    f: &[f64; N],
    j: &[[f64; N]; N],
    step: &[f64; N],
) -> E {
    (0..N).fold(E::ZERO, |total, row| {
        let row_derivative: E = (0..N).fold(E::ZERO, |sum, col| {
            sum + E::new(j[row][col]) * E::new(step[col])
        });
        total + E::new(f[row]) * row_derivative
    })
}

/// Scaled Armijo test `trial / current <= sqrt(1 + 2 c1 d / current^2)` in long double;
/// `is_armijo_accepted(1.0, 0.5, LongDouble::new(-1.0))`.
pub fn is_armijo_accepted(current_norm: f64, trial_norm: f64, directional_derivative: E) -> bool {
    // `!(x > 0)` / `!(d < 0)` also reject NaN, exactly like the native negated comparisons.
    if current_norm.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        || directional_derivative.partial_cmp(&E::ZERO) != Some(std::cmp::Ordering::Less)
    {
        return false;
    }
    let current: E = E::new(current_norm);
    let ratio: E = E::new(trial_norm) / current;
    let normalized: E = directional_derivative / current / current;
    let bound: E = E::new(1.0) + E::new(2.0) * E::new(ARMIJO_C1) * normalized;
    bound >= E::ZERO && ratio <= bound.sqrt()
}

/// True when every residual and Jacobian entry is finite; `is_finite_system_n(&f, &j)`.
pub fn is_finite_system_n<const N: usize>(f: &[f64; N], j: &[[f64; N]; N]) -> bool {
    f.iter()
        .chain(j.iter().flatten())
        .all(|value| value.is_finite())
}

/// `std::max(a, b)`: returns `a` unless `a < b`, so a NaN first operand survives;
/// `std_max(f64::NAN, 1.0).is_nan()`.
pub(super) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// `std::max({...})` over an initializer list: the first maximal element wins and a NaN
/// only survives in first position; `list_max(&[1.0, 3.0, 2.0]) == 3.0`.
pub(super) fn list_max(values: &[f64]) -> f64 {
    values[1..].iter().fold(
        values[0],
        |largest, value| if largest < *value { *value } else { largest },
    )
}
