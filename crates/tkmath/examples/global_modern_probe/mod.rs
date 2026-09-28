//! Native global strategy results and complete objective traces.
use super::swarm_probe::{Fixture, optional, vector};
use rustycad_tkmath::{math::Vector, math_opt::*, math_utils::VectorResult};
fn result(r: VectorResult, f: &mut Fixture) {
    print!("{} {} ", r.status as i32, r.iterations);
    vector(&r.solution);
    optional(r.value);
    vector(&r.gradient);
    print!("{} ", f.calls.len());
    for x in &f.calls {
        for v in x {
            print!("{} ", v.to_bits());
        }
    }
    f.calls.clear();
}
pub fn modern_global_case(v: &[f64]) {
    let n: usize = v[1] as usize;
    let mut f: Fixture = Fixture {
        mode: v[0] as i32,
        n,
        fail: v[2] as usize,
        calls: Vec::new(),
    };
    let mut cfg: GlobalConfig = GlobalConfig {
        strategy: [
            GlobalStrategy::PSO,
            GlobalStrategy::MultiStart,
            GlobalStrategy::PSOHybrid,
            GlobalStrategy::DifferentialEvolution,
        ][v[4] as usize],
        population: v[5] as usize,
        starts: v[6] as usize,
        seed: v[3] as u32,
        mutation_scale: v[8],
        crossover_prob: v[9],
        polish_budget_per_dim: v[10] as u32,
        ..Default::default()
    };
    cfg.solver.max_iterations = v[7] as u32;
    cfg.solver.tolerance = v[11];
    let lower: Vector = Vector::filled(-2, n as i32 - 3, -2.0).unwrap();
    let upper: Vector = Vector::filled(5, n as i32 + 4, 2.0).unwrap();
    result(differential_evolution(&mut f, &lower, &upper, &cfg), &mut f);
    result(multi_start(&mut f, &lower, &upper, &cfg), &mut f);
    result(global_minimum(&mut f, &lower, &upper, &cfg), &mut f);
    let mut p: PSOConfig = PSOConfig {
        particles: 4,
        omega: 1.5,
        boundary_mode: PSOBoundaryMode::Reflect,
        polish_budget_per_dim: 0,
        ..Default::default()
    };
    p.solver.max_iterations = 3;
    if v[13] != 0.0 {
        p.init_mode = PSOInitMode::SeededOnly;
    }
    let position: Vector = Vector::filled(3, n as i32 + 2, 0.25).unwrap();
    let seeds: Vec<PSOSeedParticle> = vec![PSOSeedParticle::with_value(&position, -100.0)];
    let mut stats: PSOStats = PSOStats {
        function_evals: 999,
        ..Default::default()
    };
    result(
        global_minimum_with_pso(
            &mut f,
            &lower,
            &upper,
            &cfg,
            (v[12] != 0.0).then_some(&p),
            (v[13] != 0.0).then_some(seeds.as_slice()),
            Some(&mut stats),
        ),
        &mut f,
    );
    print!(
        "{} {} {} {} {} ",
        stats.function_evals,
        stats.iterations,
        stats.boundary_corrections,
        stats.stagnation_events,
        stats.restarts
    );
    optional(stats.initial_best);
    optional(stats.final_best);
    println!();
}
