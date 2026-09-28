use rustycad_tkmath::math::{
    GlobalObjective, GlobalOptimum, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient,
    MultipleVarFunctionWithHessian, Vector,
};
struct GlobalFunction {
    n: i32,
    shape: i32,
    fail: i32,
    state: i32,
    shift: f64,
}
impl GlobalFunction {
    fn center(&self, i: i32) -> f64 {
        self.shift + 0.25 * f64::from(i)
    }
    fn objective(&self, x: &Vector<'_>) -> f64 {
        let mut sum: f64 = 0.0;
        for i in 1..=self.n {
            let t: f64 = x[i] - self.center(i);
            sum += if self.shape == 1 {
                t.sin() + 0.1 * t * t
            } else if self.shape == 2 {
                3.0
            } else {
                f64::from(i) * t * t
            };
        }
        sum
    }
    fn first_derivatives(&self, x: &Vector<'_>, g: &mut Vector<'_>) {
        for i in 1..=self.n {
            let t: f64 = x[i] - self.center(i);
            g[i] = if self.shape == 1 {
                t.cos() + 0.2 * t
            } else if self.shape == 2 {
                0.0
            } else {
                2.0 * f64::from(i) * t
            };
        }
    }
    fn trace(&self, kind: char, x: &Vector<'_>) {
        print!("{kind} ");
        print_vector(x);
    }
}
fn print_vector(x: &Vector<'_>) {
    for i in 1..=x.length() {
        print!("{} ", x[i].to_bits());
    }
}
impl MultipleVarFunction for GlobalFunction {
    fn variables(&self) -> i32 {
        self.n
    }
    fn value(&mut self, x: &Vector<'_>, v: &mut f64) -> bool {
        *v = self.objective(x);
        self.trace('v', x);
        print!("{} ", v.to_bits());
        self.fail != 3
    }
    fn state_number(&mut self) -> i32 {
        self.state += 1;
        print!("s {} ", self.state);
        self.state
    }
}
impl MultipleVarFunctionWithGradient for GlobalFunction {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        self.first_derivatives(x, g);
        self.trace('g', x);
        print_vector(g);
        self.fail != 2
    }
    fn values(&mut self, x: &Vector<'_>, v: &mut f64, g: &mut Vector<'_>) -> bool {
        *v = self.objective(x);
        self.first_derivatives(x, g);
        self.trace('c', x);
        print!("{} ", v.to_bits());
        print_vector(g);
        self.fail != 2 && self.fail != 3
    }
}
impl MultipleVarFunctionWithHessian for GlobalFunction {
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        v: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        *v = self.objective(x);
        self.first_derivatives(x, g);
        for i in 1..=self.n {
            for j in 1..=self.n {
                h[(i, j)] = 0.0;
            }
            let t: f64 = x[i] - self.center(i);
            h[(i, i)] = if self.shape == 1 {
                -t.sin() + 0.2
            } else if self.shape == 2 {
                0.0
            } else {
                2.0 * f64::from(i)
            };
        }
        self.trace('h', x);
        print!("{} ", v.to_bits());
        print_vector(g);
        for i in 1..=self.n {
            for j in 1..=self.n {
                print!("{} ", h[(i, j)].to_bits());
            }
        }
        self.fail != 1 && self.fail != 3
    }
}
fn objective(f: &mut GlobalFunction, capability: i32) -> GlobalObjective<'_> {
    match capability {
        0 => GlobalObjective::Value(f),
        1 => GlobalObjective::Gradient(f),
        _ => GlobalObjective::Hessian(f),
    }
}
fn results(s: &GlobalOptimum<'_>, n: i32) {
    let [tol, same]: [f64; 2] = s.tolerances();
    print!(
        "out {} {} {} {} {} {} {} {} ",
        i32::from(s.is_done()),
        s.minimum().to_bits(),
        s.number_of_extrema(),
        s.continuity(),
        i32::from(s.lipschitz_locked()),
        s.functional_minimal_value().to_bits(),
        tol.to_bits(),
        same.to_bits()
    );
    let mut p: Vector<'static> = Vector::filled(1, n, 91.0).unwrap();
    for i in 1..=s.number_of_extrema() {
        s.points(i, &mut p).unwrap();
        print_vector(&p);
    }
    for i in [0, s.number_of_extrema() + 1] {
        match s.points(i, &mut p) {
            Ok(()) => print_vector(&p),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
}
pub(super) fn global_case(v: &[f64]) {
    let (n, shape, fail, capability, continuity, locked, single): (
        i32,
        i32,
        i32,
        i32,
        i32,
        bool,
        bool,
    ) = (
        v[0] as i32,
        v[1] as i32,
        v[2] as i32,
        v[3] as i32,
        v[4] as i32,
        v[5] != 0.0,
        v[6] != 0.0,
    );
    let (shift, lo, hi, c, tol, same): (f64, f64, f64, f64, f64, f64) =
        (v[7], v[8], v[9], v[10], v[11], v[12]);
    let mut f: GlobalFunction = GlobalFunction {
        n,
        shape,
        fail,
        state: 0,
        shift,
    };
    let mut g: GlobalFunction = GlobalFunction {
        n,
        shape,
        fail,
        state: 0,
        shift: shift + 0.3,
    };
    let lower: Vector<'static> = Vector::filled(1, n, lo).unwrap();
    let upper: Vector<'static> = Vector::filled(1, n, hi).unwrap();
    let local_lo: Vector<'static> = Vector::filled(1, n, lo + 0.1 * (hi - lo)).unwrap();
    let local_hi: Vector<'static> = Vector::filled(1, n, hi - 0.1 * (hi - lo)).unwrap();
    let mut s: GlobalOptimum<'_> =
        GlobalOptimum::with_options(objective(&mut f, capability), &lower, &upper, c, tol, same)
            .unwrap();
    results(&s, n);
    s.set_continuity(continuity);
    s.set_lipschitz_locked(locked);
    s.set_functional_minimal_value(if shape == 2 { 3.0 * f64::from(n) } else { 0.0 });
    s.set_tolerances(tol, same);
    s.perform(single).unwrap();
    results(&s, n);
    s.set_local_params(&local_lo, &local_hi).unwrap();
    s.perform(!single).unwrap();
    results(&s, n);
    s.set_global_params(objective(&mut g, capability), &lower, &upper, c, tol, same)
        .unwrap();
    results(&s, n);
    s.perform(true).unwrap();
    results(&s, n);
    println!();
}
