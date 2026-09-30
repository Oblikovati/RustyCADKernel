//! MathOpt_PSO.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{swarm::Swarm, *};
use crate::math_utils::clamp;
impl Swarm<'_> {
    pub(super) fn initialize(
        &mut self,
        function: &mut impl MultipleVarFunction,
        seeds: &[PSOSeedParticle],
    ) {
        let direct: usize = if self.config.init_mode == PSOInitMode::RandomOnly {
            0
        } else {
            seeds.len().min(self.particles.len())
        };
        for (i, seed) in seeds.iter().take(direct).enumerate() {
            self.seed_particle(i, seed);
            self.evaluate_initial(function, i);
        }
        for i in direct..self.particles.len() {
            if self.config.init_mode == PSOInitMode::SeededOnly {
                self.jitter_particle(i, &seeds[i % direct]);
            } else {
                self.random_particle(i);
            }
            self.evaluate_initial(function, i);
        }
        self.stats.initial_best = self.best_value;
    }
    fn seed_particle(&mut self, index: usize, seed: &PSOSeedParticle) {
        let particle: &mut swarm::Particle = &mut self.particles[index];
        for i in 0..self.range.len() {
            *particle.x.change_at(i).unwrap() = clamp(
                *seed.position.at(i).unwrap(),
                *self.bounds.lower.at(i).unwrap(),
                *self.bounds.upper.at(i).unwrap(),
            );
        }
        for i in 0..self.range.len() {
            particle.velocity[i] = if let Some(v) = &seed.velocity {
                clamp(
                    *v.at(i).unwrap(),
                    -self.velocity_max[i],
                    self.velocity_max[i],
                )
            } else {
                (2.0 * self.rng.next_real() - 1.0) * self.velocity_max[i]
            };
        }
    }
    fn jitter_particle(&mut self, index: usize, seed: &PSOSeedParticle) {
        for i in 0..self.range.len() {
            let jitter: f64 = (2.0 * self.rng.next_real() - 1.0) * 0.1 * self.range[i];
            *self.particles[index].x.change_at(i).unwrap() = clamp(
                seed.position.at(i).unwrap() + jitter,
                *self.bounds.lower.at(i).unwrap(),
                *self.bounds.upper.at(i).unwrap(),
            );
            self.particles[index].velocity[i] =
                (2.0 * self.rng.next_real() - 1.0) * self.velocity_max[i];
        }
    }
    pub(super) fn random_particle(&mut self, index: usize) {
        for i in 0..self.range.len() {
            *self.particles[index].x.change_at(i).unwrap() =
                self.bounds.lower.at(i).unwrap() + self.rng.next_real() * self.range[i];
            self.particles[index].velocity[i] =
                (2.0 * self.rng.next_real() - 1.0) * self.velocity_max[i];
        }
    }
}
