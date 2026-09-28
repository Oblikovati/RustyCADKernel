//! Equation-system solvers, set integration and the linear solvers' status dumps.
use super::{dumped, guarded, reals, vector_from};
use rustycad_tkmath::math::{
    Crout, FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, GaussSetIntegration,
    Householder, IntegerVector, Matrix, NewtonFunctionSetRoot, Uzawa, Vector,
};
/// F_i(x) = x_i^2 - c_i + coupling * x_{i+1}.
struct Squares {
    fail: i32,
    calls: i32,
    coupling: f64,
    c: Vec<f64>,
}
impl Squares {
    fn residuals(&self, x: &Vector<'_>, r: &mut Vector<'_>) {
        let n = self.c.len() as i32;
        for i in 1..=n {
            let next = if i < n { self.coupling * x[i + 1] } else { 0.0 };
            r[i] = x[i] * x[i] - self.c[i as usize - 1] + next;
        }
    }
    fn jacobian(&self, x: &Vector<'_>, m: &mut Matrix<'_>) {
        let n = self.c.len() as i32;
        m.init(0.0);
        for i in 1..=n {
            m[(i, i)] = 2.0 * x[i];
            if i < n {
                m[(i, i + 1)] = self.coupling;
            }
        }
    }
    fn count(&mut self) -> bool {
        self.calls += 1;
        self.calls != self.fail
    }
}
impl FunctionSet for Squares {
    fn variables(&self) -> i32 {
        self.c.len() as i32
    }
    fn equations(&self) -> i32 {
        self.c.len() as i32
    }
    fn value(&mut self, x: &Vector<'_>, r: &mut Vector<'_>) -> bool {
        self.residuals(x, r);
        self.count()
    }
}
impl FunctionSetWithDerivatives for Squares {
    fn derivatives(&mut self, x: &Vector<'_>, m: &mut Matrix<'_>) -> bool {
        self.jacobian(x, m);
        self.count()
    }
    fn values(&mut self, x: &Vector<'_>, r: &mut Vector<'_>, m: &mut Matrix<'_>) -> bool {
        self.residuals(x, r);
        self.jacobian(x, m);
        self.count()
    }
}
/// One variable, two equations: [t^2 - c, c*t].
struct Curve {
    variables: i32,
    c: f64,
    fail: i32,
    calls: i32,
}
impl FunctionSet for Curve {
    fn variables(&self) -> i32 {
        self.variables
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, r: &mut Vector<'_>) -> bool {
        r[1] = x[1] * x[1] - self.c;
        r[2] = self.c * x[1];
        self.calls += 1;
        self.calls != self.fail
    }
}
pub(crate) fn sets_case(v: &[f64]) {
    let (n, limit, tol, cutoff) = (v[0] as usize, v[1] as i32, v[2], v[4] as i32);
    let mut f = Squares {
        fail: v[3] as i32,
        calls: 0,
        coupling: v[5],
        c: v[6..6 + n].to_vec(),
    };
    let start = vector_from(&v[6 + n..6 + 2 * n]);
    let tolerances = Vector::filled(1, n as i32, tol).unwrap();
    // Mirrors the native probe: the general set solver runs without injected failures.
    let fail = f.fail;
    f.fail = 0;
    set_roots(&mut f, &start, &tolerances, [limit, cutoff]);
    f.fail = fail;
    newton_set_roots(&mut f, &start, &tolerances, [limit, cutoff]);
    set_integration(&f, &start, limit);
    linear_dumps(&f, &start, limit);
    println!();
}
fn set_roots(
    f: &mut Squares,
    start: &Vector<'_>,
    tolerances: &Vector<'_>,
    [limit, cutoff]: [i32; 2],
) {
    f.calls = 0;
    let mut root = FunctionSetRoot::new(f, tolerances, limit).unwrap();
    root.perform(f, start, false).unwrap();
    dumped(|text| root.dump(text));
    print!("{} ", root.to_string().len());
    f.calls = 0;
    let mut tests = 0;
    let mut root = FunctionSetRoot::new(f, tolerances, limit).unwrap();
    let n = f.variables();
    let lower = Vector::filled(1, n, -f64::MAX).unwrap();
    let upper = Vector::filled(1, n, f64::MAX).unwrap();
    let mut criterion = |_: &FunctionSetRoot, _: &mut Squares| {
        tests += 1;
        tests >= cutoff
    };
    root.perform_with_criterion(f, start, [&lower, &upper], false, &mut criterion)
        .unwrap();
    dumped(|text| root.dump(text));
    print!("{tests} ");
}
fn newton_set_roots(
    f: &mut Squares,
    start: &Vector<'_>,
    tolerances: &Vector<'_>,
    [limit, cutoff]: [i32; 2],
) {
    let tol = tolerances[1];
    f.calls = 0;
    let mut root = NewtonFunctionSetRoot::new(f, tolerances, tol, limit).unwrap();
    root.perform(f, start).unwrap();
    dumped(|text| root.dump(text));
    f.calls = 0;
    let mut tests = 0;
    let mut root = NewtonFunctionSetRoot::new(f, tolerances, tol, limit).unwrap();
    let n = f.variables();
    let lower = Vector::filled(1, n, -f64::MAX).unwrap();
    let upper = Vector::filled(1, n, f64::MAX).unwrap();
    let mut criterion = |_: &NewtonFunctionSetRoot, _: &mut Squares| {
        tests += 1;
        tests >= cutoff
    };
    root.perform_with_criterion(f, start, [&lower, &upper], &mut criterion)
        .unwrap();
    dumped(|text| root.dump(text));
    print!("{tests} ");
}
fn set_integration(f: &Squares, start: &Vector<'_>, limit: i32) {
    let mut curve = Curve {
        variables: 1,
        c: f.c[0],
        fail: f.fail,
        calls: 0,
    };
    let lower = Vector::filled(1, 1, start[1]).unwrap();
    let upper = Vector::filled(1, 1, start[1] + 2.0).unwrap();
    let mut order = IntegerVector::filled(1, 1, limit % 12).unwrap();
    let integral = GaussSetIntegration::new(&mut curve, &lower, &upper, &order).unwrap();
    dumped(|text| integral.dump(text));
    if integral.is_done() {
        reals(integral.value().unwrap());
    }
    curve.variables = 2;
    guarded(|| GaussSetIntegration::new(&mut curve, &lower, &upper, &order).map(|_| ()));
    order[1] = 99;
    curve.variables = 1;
    guarded(|| GaussSetIntegration::new(&mut curve, &lower, &upper, &order).map(|_| ()));
}
fn linear_dumps(f: &Squares, start: &Vector<'_>, limit: i32) {
    let n = f.c.len() as i32;
    let mut a = Matrix::with_bounds(1, n, 1, n).unwrap();
    for i in 1..=n {
        for j in 1..=n {
            a[(i, j)] = if i == j {
                2.0 + f.c[i as usize - 1]
            } else {
                0.5 * f.coupling
            };
        }
    }
    let rhs = vector_from(&f.c);
    dumped(|text| {
        Householder::from_vector(&a, &rhs, 1e-20)
            .unwrap()
            .dump(text)
    });
    dumped(|text| Crout::with_min_pivot(&a, 1e-20).unwrap().dump(text));
    let constraints = Matrix::filled(1, 1, 1, n, 1.0).unwrap();
    let second = Vector::filled(1, 1, f.c[0]).unwrap();
    let uzawa = Uzawa::with_options(&constraints, &second, start, 1e-6, 1e-6, limit).unwrap();
    dumped(|text| uzawa.dump(text));
}
