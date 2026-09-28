// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! MathOpt_GlobOpt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
/// Native global method dispatch; `global_minimum(&mut f,&lower,&upper,&config)`.
pub fn global_minimum(
    function: &mut impl MultipleVarFunction,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    config: &GlobalConfig,
) -> VectorResult {
    global_minimum_with_pso(function, lower, upper, config, None, None, None)
}
/// Supplies optional swarm settings, seeds and counters; `global_minimum_with_pso(&mut f,&lower,&upper,&config,Some(&pso_config),None,None)`.
pub fn global_minimum_with_pso(
    function: &mut impl MultipleVarFunction,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    config: &GlobalConfig,
    pso_config: Option<&PSOConfig>,
    seeds: Option<&[PSOSeedParticle]>,
    stats: Option<&mut PSOStats>,
) -> VectorResult {
    let bounds: BoxBounds<'_> = BoxBounds { lower, upper };
    if !valid(&bounds, config) {
        return failure_result(Status::InvalidInput);
    }
    match config.strategy {
        GlobalStrategy::MultiStart => multi_start(function, lower, upper, config),
        GlobalStrategy::DifferentialEvolution => {
            differential_evolution(function, lower, upper, config)
        }
        GlobalStrategy::PSO | GlobalStrategy::PSOHybrid => {
            let cfg: PSOConfig = pso_config.copied().unwrap_or_else(|| swarm_config(config));
            let result: VectorResult = pso_seeded(function, lower, upper, &cfg, seeds, stats);
            if config.strategy == GlobalStrategy::PSO {
                result
            } else {
                hybrid(function, &bounds, config, result)
            }
        }
    }
}
fn swarm_config(config: &GlobalConfig) -> PSOConfig {
    let mut cfg: PSOConfig = PSOConfig {
        particles: config.population,
        seed: config.seed,
        polish_budget_per_dim: config.polish_budget_per_dim,
        ..Default::default()
    };
    cfg.solver.max_iterations = if config.strategy == GlobalStrategy::PSOHybrid {
        (config.solver.max_iterations / 2).max(1)
    } else {
        config.solver.max_iterations
    };
    cfg.solver.tolerance = if config.strategy == GlobalStrategy::PSOHybrid {
        config.solver.tolerance * 10.0
    } else {
        config.solver.tolerance
    };
    cfg
}
fn hybrid(
    function: &mut impl MultipleVarFunction,
    bounds: &BoxBounds<'_>,
    config: &GlobalConfig,
    mut swarm: VectorResult,
) -> VectorResult {
    if swarm.solution.is_none() || swarm.value.is_none() {
        return swarm;
    }
    let local: Config = config.local((config.solver.max_iterations / 2).max(1));
    let mut bounded: BoundedCallback<'_, '_, _> = BoundedCallback { function, bounds };
    let mut result: VectorResult = powell(&mut bounded, swarm.solution.as_ref().unwrap(), &local);
    if matches!(
        result.status,
        Status::CallbackError | Status::NumericalError
    ) {
        return result;
    }
    if !result.is_done() || result.solution.is_none() {
        return swarm;
    }
    let candidate: &mut Vector<'static> = result.solution.as_mut().unwrap();
    bounds.clamp(candidate);
    let mut value: f64 = 0.0;
    if super::super::utils::value_status(&mut bounded, candidate, &mut value) != Status::Ok {
        result.status = Status::NumericalError;
        return result;
    }
    result.value = Some(value);
    if value < swarm.value.unwrap() {
        return result;
    }
    swarm.status = Status::Ok;
    swarm
}
