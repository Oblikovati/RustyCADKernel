//! MathInteg_DoubleExp.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{
    gauss::valid_interval,
    kronrod::{empty, invalid, reversed},
};
use crate::{
    math::Function,
    math_utils::{IntegConfig, IntegResult},
};
mod dispatch;
mod refine;
mod transform;
pub use dispatch::{double_exponential, tanh_sinh_singular, tanh_sinh_with_singularity};
use transform::Transform;
/// Default callback budget of one refinement level
/// (`MathInteg::THE_DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL`);
/// `assert_eq!(DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL, 10000)`.
pub const DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL: usize = 10000;
/// Largest transformed argument sampled before `exp` overflows the weights
/// (`MathInteg::THE_DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT`);
/// `assert_eq!(DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT, 700.0)`.
pub const DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT: f64 = 700.0;
/// Most negative exp-sinh argument still sampled (`MathInteg::THE_EXP_SINH_MIN_TRANSFORM_ARGUMENT`);
/// `assert_eq!(EXP_SINH_MIN_TRANSFORM_ARGUMENT, -30.0)`.
pub const EXP_SINH_MIN_TRANSFORM_ARGUMENT: f64 = -30.0;
/// Native double-exponential options; `DoubleExpConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct DoubleExpConfig {
    /// Shared tolerances and work limit; `config.integration.tolerance`.
    pub integration: IntegConfig,
    /// Maximum refinement levels; `config.nb_levels`.
    pub nb_levels: u32,
    /// Initial transformed coordinate step; `config.step_factor`.
    pub step_factor: f64,
    /// Callback budget per level; `config.max_points_per_level`.
    pub max_points_per_level: usize,
}
impl Default for DoubleExpConfig {
    fn default() -> Self {
        Self::new(1e-10, 100)
    }
}
impl DoubleExpConfig {
    /// Retains native ten levels and half-unit step; `DoubleExpConfig::new(1e-8,100)`.
    pub fn new(tolerance: f64, max_iterations: u32) -> Self {
        Self {
            integration: IntegConfig::new(tolerance, max_iterations),
            nb_levels: 10,
            step_factor: 0.5,
            max_points_per_level: DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL,
        }
    }
    fn valid(&self) -> bool {
        self.integration.tolerance.is_finite()
            && self.integration.tolerance > 0.0
            && self.step_factor.is_finite()
            && self.step_factor > 0.0
            && self.nb_levels > 0
            && self.integration.max_iterations > 0
            && self.max_points_per_level > 0
    }
}
/// Finite interval quadrature avoiding endpoint samples; `tanh_sinh(&mut f, 0.0, 1.0, &DoubleExpConfig::default())`.
pub fn tanh_sinh(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &DoubleExpConfig,
) -> IntegResult {
    if !valid_interval(lower, upper) || !config.valid() {
        return invalid();
    }
    if lower == upper {
        return empty();
    }
    if lower > upper {
        return reversed(tanh_sinh(f, upper, lower, config));
    }
    let half: f64 = 0.5 * (upper - lower);
    refine::refine(
        f,
        Transform::Finite {
            lower,
            upper,
            mid: lower + half,
            half,
        },
        config,
    )
}
/// Semi-infinite double-exponential quadrature; `exp_sinh(&mut f, 0.0, &DoubleExpConfig::default())`.
pub fn exp_sinh(f: &mut impl Function, lower: f64, config: &DoubleExpConfig) -> IntegResult {
    if !lower.is_finite() || !config.valid() {
        return invalid();
    }
    refine::refine(f, Transform::Semi(lower), config)
}
/// Double-exponential quadrature of both infinite tails; `sinh_sinh(&mut f, &DoubleExpConfig::default())`.
pub fn sinh_sinh(f: &mut impl Function, config: &DoubleExpConfig) -> IntegResult {
    if !config.valid() {
        return invalid();
    }
    refine::refine(f, Transform::Infinite, config)
}
