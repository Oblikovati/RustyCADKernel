//! MathOpt_PSO.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod config;
pub use config::*;
mod initialize;
mod polish;
mod restart;
mod swarm;
mod update;
use super::{bounds::BoxBounds, utils::CheckedFunction};
use crate::math::{MultipleVarFunction, Vector};
use crate::math_utils::{Status, VectorResult};
pub use polish::polish_coordinate_wise;
use swarm::Swarm;
/// Native basic swarm overload; `pso(&mut function,&lower,&upper,&PSOConfig::default())`.
pub fn pso(
    function: &mut impl MultipleVarFunction,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    config: &PSOConfig,
) -> VectorResult {
    pso_seeded(function, lower, upper, config, None, None)
}
/// Native seeded swarm overload with optional counters; `pso_seeded(&mut function,&lower,&upper,&config,Some(&seeds),Some(&mut stats))`.
pub fn pso_seeded(
    function: &mut impl MultipleVarFunction,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    config: &PSOConfig,
    seeds: Option<&[PSOSeedParticle]>,
    stats: Option<&mut PSOStats>,
) -> VectorResult {
    let bounds: BoxBounds<'_> = BoxBounds { lower, upper };
    let n: usize = lower.length() as usize;
    let seeds: &[PSOSeedParticle] = seeds.unwrap_or(&[]);
    if !config.valid()
        || !bounds.valid(n as i32)
        || config.particles.checked_mul(n).is_none()
        || seeds.iter().any(|s| !s.valid(n))
        || (config.init_mode == PSOInitMode::SeededOnly && seeds.is_empty())
    {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut checked: CheckedFunction<'_, _> = CheckedFunction::new(function);
    let mut swarm: Swarm = Swarm::new(&bounds, config);
    swarm.initialize(&mut checked, seeds);
    let result: VectorResult = if swarm.best_value.is_none() {
        VectorResult {
            status: if checked.failure_status == Status::Ok {
                Status::NumericalError
            } else {
                checked.failure_status
            },
            ..Default::default()
        }
    } else {
        swarm.run(&mut checked)
    };
    if let Some(output) = stats {
        *output = swarm.stats;
    }
    result
}
