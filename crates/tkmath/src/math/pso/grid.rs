use super::{Failure, MultipleVarFunction, Pso, PsoParticlesPool, Vector};
impl<F: MultipleVarFunction> Pso<'_, F> {
    pub(super) fn populate(&mut self, pool: &mut PsoParticlesPool) -> Result<(), Failure> {
        let (lower, upper): (Vector<'static>, Vector<'static>) = self.interior()?;
        let mut point: Vector<'static> = lower.clone();
        let mut value: f64 = 0.0;
        let mut worst: usize = pool.worst_index();
        loop {
            self.function.value(&point, &mut value);
            if value < pool.particles[worst].distance {
                let particle: &mut super::PsoParticle = &mut pool.particles[worst];
                for i in 0..self.dimension as usize {
                    particle.position[i] = point[i as i32 + 1];
                    particle.best_position[i] = point[i as i32 + 1];
                }
                particle.distance = value;
                particle.best_distance = value;
                worst = pool.worst_index();
            }
            self.next_grid_point(&mut point, &lower, &upper);
            if point[self.dimension] > upper[self.dimension] {
                break;
            }
        }
        Ok(())
    }
    fn next_grid_point(&self, point: &mut Vector<'_>, lower: &Vector<'_>, upper: &Vector<'_>) {
        point[1] += if self.steps[1] < 1e-15 {
            1e-15
        } else {
            self.steps[1]
        };
        for dimension in 1..self.dimension {
            if point[dimension].partial_cmp(&upper[dimension]) != Some(std::cmp::Ordering::Greater)
            {
                break;
            }
            point[dimension] = lower[dimension];
            point[dimension + 1] += self.steps[dimension + 1];
        }
    }
}
