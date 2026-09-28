//! Particle initialization, first-tie selection, generator reset and native default iterations.
use rustycad_tkmath::math::{BullardGenerator, MultipleVarFunction, Pso, PsoParticlesPool, Vector};
struct CountingBowl {
    calls: usize,
}
impl MultipleVarFunction for CountingBowl {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        self.calls += 1;
        *value = (x[1] - 1.0) * (x[1] - 1.0);
        false
    }
}
#[test]
fn pool_starts_zeroed_and_selects_first_equal_distance() {
    let mut pool: PsoParticlesPool = PsoParticlesPool::new(3, 2).unwrap();
    for index in 1..=3 {
        let p = pool.particle(index).unwrap();
        assert_eq!(p.position, [0.0, 0.0]);
        assert_eq!(p.velocity, [0.0, 0.0]);
        assert_eq!(p.best_position, [0.0, 0.0]);
        assert_eq!(p.distance, f64::MAX);
        assert_eq!(p.best_distance, f64::MAX);
    }
    pool.best_particle().unwrap().position[0] = 7.0;
    assert_eq!(pool.particle(1).unwrap().position[0], 7.0);
    pool.worst_particle().unwrap().distance = 2.0;
    assert_eq!(pool.particle(1).unwrap().distance, 2.0);
    pool.particle_mut(2).unwrap().distance = 2.0;
    assert_eq!(pool.best_particle().unwrap().position[0], 7.0);
    assert!(pool.particle(0).is_err());
    assert!(pool.particle_mut(4).is_err());
}
#[test]
fn bullard_reset_reproduces_wrapping_sequence() {
    let mut first: BullardGenerator = BullardGenerator::default();
    let mut second: BullardGenerator = BullardGenerator::new(1);
    for _ in 0..100 {
        assert_eq!(first.next_int(), second.next_int());
    }
    first.set_seed(1);
    second.set_seed(1);
    assert_eq!(first.next_real().to_bits(), second.next_real().to_bits());
    let mut edge: BullardGenerator = BullardGenerator::new(u32::MAX);
    for _ in 0..100 {
        let value: f64 = edge.next_real();
        assert!((0.0..=1.0).contains(&value));
    }
}
#[test]
fn supplied_pool_keeps_initial_global_best_when_no_iterations_run() {
    let mut f: CountingBowl = CountingBowl { calls: 0 };
    let low: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let high: Vector<'static> = Vector::filled(1, 1, 2.0).unwrap();
    let steps: Vector<'static> = Vector::filled(1, 1, 0.5).unwrap();
    let mut solver: Pso<'_, CountingBowl> = Pso::new(&mut f, &low, &high, &steps).unwrap();
    let mut pool: PsoParticlesPool = PsoParticlesPool::new(2, 1).unwrap();
    let mut value: f64 = 0.0;
    let mut point: Vector<'static> = low.clone();
    solver
        .perform_particles_with_iterations(&mut pool, 2, &mut value, &mut point, 1)
        .unwrap();
    assert_eq!(value, f64::MAX);
    assert_eq!(point[1], 0.0);
    assert_ne!(pool.particle(1).unwrap().velocity[0], 0.0);
    assert_eq!(f.calls, 0);
}
#[test]
fn callback_false_is_ignored_and_default_perform_overrides_constructor_limit() {
    let mut f: CountingBowl = CountingBowl { calls: 0 };
    let low: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let high: Vector<'static> = Vector::filled(1, 1, 2.0).unwrap();
    let steps: Vector<'static> = Vector::filled(1, 1, 0.5).unwrap();
    let mut solver: Pso<'_, CountingBowl> =
        Pso::with_options(&mut f, &low, &high, &steps, 3, 1).unwrap();
    let mut pool: PsoParticlesPool = PsoParticlesPool::new(3, 1).unwrap();
    let mut value: f64 = 0.0;
    let mut point: Vector<'static> = low.clone();
    solver
        .perform_particles(&mut pool, 3, &mut value, &mut point)
        .unwrap();
    assert!(value < 1.0);
    assert!(point[1] > 0.0);
    assert!(f.calls > 0);
}
