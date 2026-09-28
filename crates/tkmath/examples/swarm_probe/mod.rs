//! Native seeded PSO result, counters, and complete objective trace.
use rustycad_tkmath::{
    math::{MultipleVarFunction, Vector},
    math_opt::*,
    math_utils::VectorResult,
};
pub(super) struct Fixture {
    pub(super) mode: i32,
    pub(super) n: usize,
    pub(super) fail: usize,
    pub(super) calls: Vec<Vec<f64>>,
}
impl MultipleVarFunction for Fixture {
    fn variables(&self) -> i32 {
        self.n as i32
    }
    fn value(&mut self, x: &Vector<'_>, out: &mut f64) -> bool {
        let point: Vec<f64> = (0..self.n).map(|i| *x.at(i).unwrap()).collect();
        self.calls.push(point);
        if self.calls.len() == self.fail
            || self.mode == 2
            || (self.mode == 4 && *x.at(0).unwrap() < 0.0)
        {
            return false;
        }
        if self.mode == 3 {
            *out = f64::NAN;
            return true;
        }
        if self.mode == 6 && self.calls.len().is_multiple_of(3) {
            return true;
        }
        *out = 0.0;
        if self.mode == 5 {
            *out = 1.0;
            return true;
        }
        if self.mode == 1 && self.n >= 2 {
            let a: f64 = *x.at(0).unwrap();
            let b: f64 = *x.at(1).unwrap();
            *out = 100.0 * (b - a * a) * (b - a * a) + (1.0 - a) * (1.0 - a);
            return true;
        }
        for i in 0..self.n {
            let d: f64 = x.at(i).unwrap() - 0.25;
            *out += d * d;
        }
        true
    }
}
pub(super) fn optional(v: Option<f64>) {
    print!("{} ", u8::from(v.is_some()));
    if let Some(x) = v {
        print!("{} ", x.to_bits());
    }
}
pub(super) fn vector(v: &Option<Vector<'_>>) {
    print!("{} ", u8::from(v.is_some()));
    if let Some(x) = v {
        print!("{} {} ", x.lower(), x.length());
        for i in 0..x.length() as usize {
            print!("{} ", x.at(i).unwrap().to_bits());
        }
    }
}
fn seeds(v: &[f64], n: usize) -> Vec<PSOSeedParticle> {
    let kind: i32 = v[19] as i32;
    let count: usize = if kind == 0 {
        0
    } else if kind == 2 {
        3
    } else {
        1
    };
    (0..count)
        .map(|k| {
            let dims: usize = if kind == 6 {
                0
            } else if kind == 3 {
                n + 1
            } else {
                n
            };
            let p: Vector = Vector::filled(7, 6 + dims as i32, v[21] + k as f64).unwrap();
            let mut s: PSOSeedParticle = PSOSeedParticle::with_value(&p, -100.0);
            if kind == 4 {
                s.value = Some(f64::NAN);
            }
            if v[20] != 0.0 || kind == 5 {
                let mut velocity: Vector = Vector::filled(-1, n as i32 - 2, 3.0).unwrap();
                if kind == 5 {
                    *velocity.change_at(0).unwrap() = f64::INFINITY;
                }
                s.velocity = Some(velocity);
            }
            if kind == 7 {
                s.velocity = Some(Vector::with_size(n + 1));
            }
            s
        })
        .collect()
}
pub fn modern_swarm_case(v: &[f64]) {
    let n: usize = v[1] as usize;
    let mut f: Fixture = Fixture {
        mode: v[0] as i32,
        n,
        fail: v[2] as usize,
        calls: Vec::new(),
    };
    let mut cfg: PSOConfig = PSOConfig {
        seed: v[3] as u32,
        particles: v[4] as usize,
        ..Default::default()
    };
    cfg.solver.max_iterations = v[5] as u32;
    cfg.boundary_mode = [
        PSOBoundaryMode::Clamp,
        PSOBoundaryMode::Reflect,
        PSOBoundaryMode::Wrap,
    ][v[6] as usize];
    cfg.init_mode = [
        PSOInitMode::RandomOnly,
        PSOInitMode::SeededOnly,
        PSOInitMode::SeededPlusRandom,
    ][v[7] as usize];
    cfg.inertia_schedule = if v[8] == 0.0 {
        PSOInertiaSchedule::Constant
    } else {
        PSOInertiaSchedule::LinearDecay
    };
    cfg.omega = v[9];
    cfg.velocity_clamp = v[10];
    cfg.solver.tolerance = v[11];
    cfg.min_iterations = v[12] as u32;
    cfg.no_improve_iters = v[13] as u32;
    cfg.restart_fraction = v[14];
    cfg.max_restarts = v[15] as u32;
    cfg.polish_budget_per_dim = v[16] as u32;
    cfg.target_value = (v[17] != 0.0).then_some(v[18]);
    let lower: Vector = Vector::filled(-3, n as i32 - 4, -2.0).unwrap();
    let upper: Vector = Vector::filled(5, n as i32 + 4, 2.0).unwrap();
    let seeds: Vec<PSOSeedParticle> = seeds(v, n);
    let mut stats: PSOStats = PSOStats {
        function_evals: 999,
        ..Default::default()
    };
    let r: VectorResult = pso_seeded(&mut f, &lower, &upper, &cfg, Some(&seeds), Some(&mut stats));
    print!("{} {} ", r.status as i32, r.iterations);
    vector(&r.solution);
    optional(r.value);
    vector(&r.gradient);
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
    print!("{} ", f.calls.len());
    for point in f.calls {
        for x in point {
            print!("{} ", x.to_bits());
        }
    }
    println!();
}
