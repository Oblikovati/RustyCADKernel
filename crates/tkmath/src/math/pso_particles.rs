//! Owned particle vectors and native first-tie distance selection.
use rustycad_tkernel::standard::Failure;
/// A particle's mutable state; access with `pool.particle_mut(1)?`.
pub struct PsoParticle {
    /// Current position, indexed from zero; `particle.position[0]`.
    pub position: Vec<f64>,
    /// Current velocity; `particle.velocity[0]`.
    pub velocity: Vec<f64>,
    /// Best recorded position; `particle.best_position[0]`.
    pub best_position: Vec<f64>,
    /// Current objective, initially RealLast; `particle.distance`.
    pub distance: f64,
    /// Best objective, initially RealLast; `particle.best_distance`.
    pub best_distance: f64,
}
/// `PSO_Particle()`: a detached particle with no vector storage (the native null
/// pointers) and `RealLast` distances; `PsoParticle::default().distance == f64::MAX`.
impl Default for PsoParticle {
    fn default() -> Self {
        Self::new(0)
    }
}
impl PsoParticle {
    /// Native `operator<`, ordering particles by current distance only;
    /// `PsoParticle::default().less(&other)`.
    pub fn less(&self, other: &Self) -> bool {
        self.distance < other.distance
    }
    fn new(dimension: usize) -> Self {
        Self {
            position: vec![0.0; dimension],
            velocity: vec![0.0; dimension],
            best_position: vec![0.0; dimension],
            distance: f64::MAX,
            best_distance: f64::MAX,
        }
    }
}
/// One-based particle pool with zeroed vectors; `PsoParticlesPool::new(32,2)?`.
pub struct PsoParticlesPool {
    pub(super) particles: Vec<PsoParticle>,
}
impl PsoParticlesPool {
    /// Allocates initialized native particle state; `PsoParticlesPool::new(32,2)?`.
    ///
    /// An empty pool is valid natively. A negative count fails the `NCollection_Array1`
    /// bounds (`Standard_RangeError`); particles without dimensions fail when their
    /// pointers into the empty memory block are taken (`Standard_OutOfRange`).
    pub fn new(count: i32, dimension: i32) -> Result<Self, Failure> {
        if count < 0 {
            return Err(Failure::with_type(
                "Standard_RangeError",
                &format!("pool of {count} particles; expected a non-negative count"),
                "",
            ));
        }
        if count > 0 && dimension < 0 {
            return Err(Failure::with_type(
                "Standard_RangeError",
                &format!("pool {count} x {dimension}; expected a non-negative dimension"),
                "",
            ));
        }
        if count > 0 && dimension == 0 {
            return Err(Failure::out_of_range(&format!(
                "pool {count} x 0; particles need at least one dimension"
            )));
        }
        Ok(Self {
            particles: (0..count)
                .map(|_| PsoParticle::new(dimension as usize))
                .collect(),
        })
    }
    /// Reads a one-based particle; `pool.particle(1)?`.
    pub fn particle(&self, index: i32) -> Result<&PsoParticle, Failure> {
        self.particles
            .get(index.wrapping_sub(1) as usize)
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "particle {index}, expected 1..={}",
                    self.particles.len()
                ))
            })
    }
    /// Mutates a one-based particle; `pool.particle_mut(1)?`.
    pub fn particle_mut(&mut self, index: i32) -> Result<&mut PsoParticle, Failure> {
        let count: usize = self.particles.len();
        self.particles
            .get_mut(index.wrapping_sub(1) as usize)
            .ok_or_else(|| {
                Failure::out_of_range(&format!("particle {index}, expected 1..={count}"))
            })
    }
    pub(super) fn best_index(&self) -> usize {
        self.extreme(false)
    }
    pub(super) fn worst_index(&self) -> usize {
        self.extreme(true)
    }
    fn extreme(&self, worst: bool) -> usize {
        let mut chosen: usize = 0;
        for index in 1..self.particles.len() {
            // std::min_element / std::max_element keep the first extreme under operator<.
            let replace: bool = if worst {
                self.particles[chosen].less(&self.particles[index])
            } else {
                self.particles[index].less(&self.particles[chosen])
            };
            if replace {
                chosen = index;
            }
        }
        chosen
    }
    /// Returns the first particle with minimal distance; an empty pool (where native code
    /// dereferences `end()`) raises `Standard_OutOfRange`. `pool.best_particle()?`.
    pub fn best_particle(&mut self) -> Result<&mut PsoParticle, Failure> {
        self.check_not_empty()?;
        let index: usize = self.best_index();
        Ok(&mut self.particles[index])
    }
    /// Returns the first particle with maximal distance; an empty pool raises
    /// `Standard_OutOfRange`. `pool.worst_particle()?`.
    pub fn worst_particle(&mut self) -> Result<&mut PsoParticle, Failure> {
        self.check_not_empty()?;
        let index: usize = self.worst_index();
        Ok(&mut self.particles[index])
    }
    fn check_not_empty(&self) -> Result<(), Failure> {
        if self.particles.is_empty() {
            return Err(Failure::out_of_range(
                "empty particle pool has no extreme particle",
            ));
        }
        Ok(())
    }
}
