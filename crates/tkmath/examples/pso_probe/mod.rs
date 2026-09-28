use rustycad_tkmath::math::{BullardGenerator, MultipleVarFunction, Pso, PsoParticlesPool, Vector};
struct SwarmFunction {
    dimension: i32,
    mode: i32,
}
impl MultipleVarFunction for SwarmFunction {
    fn variables(&self) -> i32 {
        self.dimension
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        *value = 0.0;
        for i in 1..=self.dimension {
            let t: f64 = x[i] - 0.25 * f64::from(i);
            *value += f64::from(i) * t * t;
        }
        if self.mode == 1 {
            *value = 3.0;
        }
        let good: bool = self.mode != 2 || x[1] > 0.0;
        print!("v ");
        for i in 1..=self.dimension {
            print!("{} ", x[i].to_bits());
        }
        print!("{} {} ", value.to_bits(), i32::from(good));
        good
    }
}
fn output(value: f64, point: &Vector<'_>) {
    print!("out {} ", value.to_bits());
    for i in 1..=point.length() {
        print!("{} ", point[i].to_bits());
    }
}
fn pool_output(pool: &mut PsoParticlesPool, count: i32, dimension: i32) {
    for i in 1..=count {
        let p = pool.particle(i).unwrap();
        print!("{} {} ", p.distance.to_bits(), p.best_distance.to_bits());
        for j in 0..dimension as usize {
            print!(
                "{} {} {} ",
                p.position[j].to_bits(),
                p.velocity[j].to_bits(),
                p.best_position[j].to_bits()
            );
        }
    }
    print!(
        "{} {} ",
        pool.best_particle().distance.to_bits(),
        pool.worst_particle().distance.to_bits()
    );
}
pub(super) fn pso_case(v: &[f64]) {
    let (n, mode, count, limit): (i32, i32, i32, i32) =
        (v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32);
    let (lower, upper, step): (f64, f64, f64) = (v[4], v[5], v[6]);
    let mut f: SwarmFunction = SwarmFunction { dimension: n, mode };
    let mut low: Vector<'static> = Vector::with_bounds(1, n).unwrap();
    let mut high: Vector<'static> = low.clone();
    let mut steps: Vector<'static> = low.clone();
    let mut point: Vector<'static> = low.clone();
    let mut value: f64 = 71.0;
    for i in 1..=n {
        low[i] = lower - 0.1 * f64::from(i);
        high[i] = upper + 0.2 * f64::from(i);
        steps[i] = step + 0.07 * f64::from(i);
    }
    let mut solver: Pso<'_, SwarmFunction> =
        Pso::with_options(&mut f, &low, &high, &steps, count, 7).unwrap();
    solver
        .perform_with_iterations(&steps, &mut value, &mut point, limit)
        .unwrap();
    output(value, &point);
    let mut pool: PsoParticlesPool = PsoParticlesPool::new(count + 1, n).unwrap();
    pool_output(&mut pool, count + 1, n);
    // Pool initialization uses an equivalent stateless objective while the solver retains its mutable borrow.
    initialize_pool(&mut pool, count + 1, n, lower, upper, mode, &mut point);
    solver
        .perform_particles_with_iterations(&mut pool, count, &mut value, &mut point, limit)
        .unwrap();
    output(value, &point);
    pool_output(&mut pool, count + 1, n);
    for i in 1..=n {
        steps[i] *= 1.3;
    }
    solver
        .perform_with_iterations(&steps, &mut value, &mut point, limit)
        .unwrap();
    output(value, &point);
    println!();
}
fn initialize_pool(
    pool: &mut PsoParticlesPool,
    count: i32,
    n: i32,
    lower: f64,
    upper: f64,
    mode: i32,
    point: &mut Vector<'_>,
) {
    let mut f: SwarmFunction = SwarmFunction { dimension: n, mode };
    for i in 1..=count {
        let p = pool.particle_mut(i).unwrap();
        for j in 0..n as usize {
            p.position[j] = lower + (upper - lower) * (0.1 + 0.13 * f64::from(i)) + 0.2 * j as f64;
            p.best_position[j] = p.position[j];
            point[j as i32 + 1] = p.position[j];
        }
        f.value(point, &mut p.distance);
        p.best_distance = p.distance;
    }
}
pub(super) fn bullard_case(v: &[f64]) {
    let mut random: BullardGenerator = BullardGenerator::new(v[0] as u32);
    for _ in 0..32 {
        print!("{} {} ", random.next_int(), random.next_real().to_bits());
    }
    random.set_seed(1);
    println!("{} {} ", random.next_int(), random.next_real().to_bits());
}
