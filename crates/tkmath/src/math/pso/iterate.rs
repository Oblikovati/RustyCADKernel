use super::{BullardGenerator, Failure, MultipleVarFunction, Pso, PsoParticlesPool, Vector};
struct Swarm {
    lower: Vector<'static>,
    upper: Vector<'static>,
    best: Vector<'static>,
    distance: f64,
    termination: Vector<'static>,
    random: BullardGenerator,
}
impl<F: MultipleVarFunction> Pso<'_, F> {
    /// Updates pool state in place and overwrites the output; `pso.perform_particles_with_iterations(&mut pool,n,&mut value,&mut point,30)?`.
    pub fn perform_particles_with_iterations(
        &mut self,
        pool: &mut PsoParticlesPool,
        count: i32,
        value: &mut f64,
        point: &mut Vector<'_>,
        limit: i32,
    ) -> Result<(), Failure> {
        let (lower, upper): (Vector<'static>, Vector<'static>) = self.interior()?;
        self.limit = limit;
        self.count = count;
        let mut random: BullardGenerator = BullardGenerator::default();
        self.initial_velocities(pool, &mut random)?;
        let particle: &super::PsoParticle = pool.particle(pool.best_index() as i32 + 1)?;
        let mut best: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        for i in 0..self.dimension as usize {
            best[i as i32 + 1] = particle.position[i];
        }
        let mut swarm: Swarm = Swarm {
            lower,
            upper,
            best,
            distance: particle.distance,
            termination: self.steps.divided(2048.0)?,
            random,
        };
        for step in 1..self.limit {
            if self.iteration(pool, &mut swarm, step)? {
                break;
            }
        }
        *value = swarm.distance;
        point.initialized(&swarm.best)?;
        Ok(())
    }
    fn iteration(
        &mut self,
        pool: &mut PsoParticlesPool,
        swarm: &mut Swarm,
        step: i32,
    ) -> Result<bool, Failure> {
        let mut minimum: Vector<'static> = Vector::filled(1, self.dimension, f64::MAX)?;
        let mut current: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        for index in 1..=self.count {
            let random: [f64; 2] = [swarm.random.next_real(), swarm.random.next_real()];
            let particle: &mut super::PsoParticle = pool.particle_mut(index)?;
            Self::move_particle(particle, swarm, random, &mut current, &mut minimum);
            self.function.value(&current, &mut particle.distance);
            Self::update_best(particle, swarm);
        }
        if (1..=self.dimension).any(|i| minimum[i] > swarm.termination[i]) {
            return Ok(false);
        }
        if step > 16 {
            return Ok(true);
        }
        self.restart_velocities(pool, &swarm.lower, &swarm.upper, &mut swarm.random)?;
        Ok(false)
    }
    fn move_particle(
        particle: &mut super::PsoParticle,
        swarm: &Swarm,
        random: [f64; 2],
        current: &mut Vector<'_>,
        minimum: &mut Vector<'_>,
    ) {
        for i in 0..current.length() as usize {
            particle.velocity[i] = particle.velocity[i] * 0.72900
                + (particle.best_position[i] - particle.position[i]) * (1.49445 * random[0])
                + (swarm.best[i as i32 + 1] - particle.position[i]) * (1.49445 * random[1]);
            particle.position[i] += particle.velocity[i];
            let low: f64 = swarm.lower[i as i32 + 1];
            let high: f64 = swarm.upper[i as i32 + 1];
            if particle.position[i] < low {
                particle.position[i] = low;
            }
            if high < particle.position[i] {
                particle.position[i] = high;
            }
            current[i as i32 + 1] = particle.position[i];
            let velocity: f64 = particle.velocity[i].abs();
            if minimum[i as i32 + 1].partial_cmp(&velocity) != Some(std::cmp::Ordering::Less) {
                minimum[i as i32 + 1] = velocity;
            }
        }
    }
    fn update_best(particle: &mut super::PsoParticle, swarm: &mut Swarm) {
        if particle.distance >= particle.best_distance
            || particle.distance.is_nan()
            || particle.best_distance.is_nan()
        {
            return;
        }
        particle.best_distance = particle.distance;
        for i in 0..swarm.best.length() as usize {
            particle.best_position[i] = particle.position[i];
        }
        if particle.distance < swarm.distance {
            swarm.distance = particle.distance;
            for i in 0..swarm.best.length() as usize {
                swarm.best[i as i32 + 1] = particle.position[i];
            }
        }
    }
}
