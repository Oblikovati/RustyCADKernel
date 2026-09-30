//! MathOpt_PSO.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use crate::math_utils::RandomGenerator;
pub(super) struct Particle {
    pub x: Vector<'static>,
    pub velocity: Vec<f64>,
    pub best: Vector<'static>,
    pub best_value: Option<f64>,
}
pub(super) struct Swarm<'a> {
    pub bounds: &'a BoxBounds<'a>,
    pub config: &'a PSOConfig,
    pub particles: Vec<Particle>,
    pub rng: RandomGenerator,
    pub range: Vec<f64>,
    pub velocity_max: Vec<f64>,
    pub best: Vector<'static>,
    pub best_value: Option<f64>,
    pub stats: PSOStats,
}
impl<'a> Swarm<'a> {
    pub fn new(bounds: &'a BoxBounds<'a>, config: &'a PSOConfig) -> Self {
        let n: usize = bounds.lower.length() as usize;
        let range: Vec<f64> = (0..n)
            .map(|i| bounds.upper.at(i).unwrap() - bounds.lower.at(i).unwrap())
            .collect();
        let velocity_max: Vec<f64> = range.iter().map(|v| config.velocity_clamp * v).collect();
        let particles: Vec<Particle> = (0..config.particles)
            .map(|_| Particle {
                x: Vector::with_size(n),
                velocity: vec![0.0; n],
                best: Vector::with_size(n),
                best_value: None,
            })
            .collect();
        Self {
            bounds,
            config,
            particles,
            rng: RandomGenerator::new(config.seed as u64),
            range,
            velocity_max,
            best: Vector::with_size(n),
            best_value: None,
            stats: PSOStats::default(),
        }
    }
    pub fn evaluate_initial(&mut self, function: &mut impl MultipleVarFunction, index: usize) {
        let mut value: f64 = 0.0;
        let particle: &mut Particle = &mut self.particles[index];
        particle.best_value = if function.value(&particle.x, &mut value) {
            Some(value)
        } else {
            None
        };
        particle.best.initialized(&particle.x).unwrap();
        self.stats.function_evals += 1;
        self.improve_global(index);
    }
    pub fn improve_global(&mut self, index: usize) {
        let particle: &Particle = &self.particles[index];
        if let Some(value) = particle
            .best_value
            .filter(|v| self.best_value.is_none_or(|old| *v < old))
        {
            self.best_value = Some(value);
            self.best.initialized(&particle.best).unwrap();
        }
    }
    pub fn run(&mut self, function: &mut impl MultipleVarFunction) -> VectorResult {
        let mut previous: f64 = self.best_value.unwrap();
        let mut stagnation: u32 = 0;
        let mut converged: bool = false;
        for iteration in 0..self.config.solver.max_iterations {
            self.stats.iterations = iteration + 1;
            self.update_all(function, iteration);
            if self
                .config
                .target_value
                .is_some_and(|target| self.best_value.unwrap() <= target)
                || self.stagnated(function, previous, &mut stagnation)
            {
                converged = true;
                break;
            }
            previous = self.best_value.unwrap();
        }
        let mut polish_evals: usize = 0;
        if self.config.polish_budget_per_dim > 0 {
            polish_coordinate_wise(
                function,
                &mut self.best,
                self.best_value.as_mut().unwrap(),
                self.bounds.lower,
                self.bounds.upper,
                self.config.solver.tolerance,
                self.config.polish_budget_per_dim as usize * self.range.len(),
                &mut polish_evals,
            );
        }
        self.stats.function_evals += polish_evals;
        self.stats.final_best = self.best_value;
        VectorResult {
            status: if converged {
                Status::Ok
            } else {
                Status::MaxIterations
            },
            iterations: self.stats.iterations,
            solution: Some(self.best.clone()),
            value: self.best_value,
            ..Default::default()
        }
    }
}
