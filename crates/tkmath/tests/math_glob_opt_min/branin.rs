// OCC25004: Branin's three global minima with native Hessian arithmetic.
struct BraninFunction {
    a: f64,
    b: f64,
    c: f64,
    r: f64,
    s: f64,
    t: f64,
}
impl BraninFunction {
    fn new() -> Self {
        Self {
            a: 1.0,
            b: 5.1 / (4.0 * PI * PI),
            c: 5.0 / PI,
            r: 6.0,
            s: 10.0,
            t: 1.0 / (8.0 * PI),
        }
    }
}
impl MultipleVarFunction for BraninFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        let (u, v): (f64, f64) = (x[1], x[2]);
        let square: f64 = v - self.b * u * u + self.c * u - self.r;
        let linear: f64 = self.s * (1.0 - self.t) * u.cos();
        *f = self.a * square * square + linear + self.s;
        true
    }
}
impl MultipleVarFunctionWithGradient for BraninFunction {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        let (u, v): (f64, f64) = (x[1], x[2]);
        let square: f64 = v - self.b * u * u + self.c * u - self.r;
        g[1] =
            2.0 * self.a * square * (self.c - 2.0 * self.b * u) - self.s * (1.0 - self.t) * u.sin();
        g[2] = 2.0 * self.a * square;
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f);
        self.gradient(x, g);
        true
    }
}
impl MultipleVarFunctionWithHessian for BraninFunction {
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        f: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        self.value(x, f);
        self.gradient(x, g);
        let (u, v): (f64, f64) = (x[1], x[2]);
        let square: f64 = v - self.b * u * u + self.c * u - self.r;
        let temporary: f64 = self.c - 2.0 * self.b * u;
        h[(1, 1)] = 2.0 * self.a * temporary * temporary
            - 4.0 * self.a * self.b * square
            - self.s * (1.0 - self.t) * u.cos();
        h[(1, 2)] = 2.0 * self.a * temporary;
        h[(2, 1)] = h[(1, 2)];
        h[(2, 2)] = 2.0 * self.a;
        true
    }
}
fn grid_point(index: i32, lower: &Vector<'_>, upper: &Vector<'_>) -> Vector<'static> {
    let i: i32 = (index - 1) / 16 + 1;
    let j: i32 = (index - 1) % 16 + 1;
    let mut point: Vector<'static> = Vector::with_bounds(1, 2).unwrap();
    point[1] = lower[1] + (upper[1] - lower[1]) * f64::from(i - 1) / (16.0 - 1.0);
    point[2] = lower[2] + (upper[2] - lower[2]) * f64::from(j - 1) / (16.0 - 1.0);
    point
}
fn branin_lipschitz(f: &mut BraninFunction, lower: &Vector<'_>, upper: &Vector<'_>) -> f64 {
    let mut values: Vector<'static> = Vector::with_bounds(1, 256).unwrap();
    for index in 1..=256 {
        f.value(&grid_point(index, lower, upper), &mut values[index]);
    }
    let mut constant: f64 = 0.0;
    // Flattening each (i,j) pair keeps the original four-loop traversal order.
    for first in 1..=256 {
        constant = branin_grid_row(first, &values, lower, upper, constant);
    }
    constant
}
fn branin_grid_row(
    first: i32,
    values: &Vector<'_>,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    mut constant: f64,
) -> f64 {
    for second in 1..=256 {
        if first == second {
            continue;
        }
        let mut a: Vector<'static> = grid_point(first, lower, upper);
        let b: Vector<'static> = grid_point(second, lower, upper);
        for i in 1..=2 {
            a[i] += -b[i];
        }
        let distance: f64 = a.norm().unwrap();
        if distance > 0.0 {
            let candidate: f64 = (values[first] - values[second]).abs() / distance;
            if candidate > constant {
                constant = candidate;
            }
        }
    }
    constant
}
#[test]
fn MathGlobOptMinTest__OCC25004_BraninFunctionGlobalOptimum() {
    let mut f: BraninFunction = BraninFunction::new();
    let mut lower: Vector<'static> = Vector::with_bounds(1, 2).unwrap();
    let mut upper: Vector<'static> = lower.clone();
    lower[1] = -5.0;
    lower[2] = 0.0;
    upper[1] = 10.0;
    upper[2] = 15.0;
    let constant: f64 = branin_lipschitz(&mut f, &lower, &upper);
    let mut solver: GlobalOptimum<'_> = GlobalOptimum::with_options(
        GlobalObjective::Hessian(&mut f),
        &lower,
        &upper,
        constant,
        1e-2,
        1e-7,
    )
    .unwrap();
    solver.perform(false).unwrap();
    assert!(solver.is_done());
    near(solver.minimum(), 0.39788735772, 0.1);
    assert_eq!(solver.number_of_extrema(), 3);
}
