//! MathOpt_GlobOpt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod config;
pub use config::{GlobalConfig, GlobalStrategy};
mod differential;
pub use differential::differential_evolution;
mod multistart;
pub use multistart::multi_start;
mod dispatch;
use super::{
    bounds::BoxBounds,
    utils::{BoundedFunction, CheckedFunction, is_valid_config},
    *,
};
use crate::math_utils::{RandomGenerator, clamp};
pub use dispatch::{global_minimum, global_minimum_with_pso};
fn failure_result(status: Status) -> VectorResult {
    VectorResult {
        status,
        ..Default::default()
    }
}
fn valid(bounds: &BoxBounds<'_>, config: &GlobalConfig) -> bool {
    is_valid_config(&config.solver) && bounds.valid(bounds.lower.length())
}
fn random_point(bounds: &BoxBounds<'_>, rng: &mut RandomGenerator) -> Vector<'static> {
    let mut point: Vector<'static> = Vector::with_size(bounds.lower.length() as usize);
    for i in 0..point.length() as usize {
        let random: f64 = rng.next_real();
        *point.change_at(i).unwrap() = bounds.lower.at(i).unwrap()
            + random * (bounds.upper.at(i).unwrap() - bounds.lower.at(i).unwrap());
    }
    point
}
