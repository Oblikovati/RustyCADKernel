//! Gradient, direction-set and Newton minimizers with default and replaced convergence tests.
use super::{dumped, flat, vector_from};
use rustycad_tkmath::math::{
    Bfgs, Frpr, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient,
    MultipleVarFunctionWithHessian, NewtonMinimum, Powell, Vector,
};
/// Separable quadratic (optionally quartic) objective; call `fail` returns false.
struct Bowl {
    quartic: bool,
    fail: i32,
    calls: i32,
    centers: Vec<f64>,
    weights: Vec<f64>,
}
impl Bowl {
    fn count(&mut self) -> bool {
        self.calls += 1;
        self.calls != self.fail
    }
    fn delta(&self, x: &Vector<'_>, i: usize) -> f64 {
        x[i as i32 + 1] - self.centers[i]
    }
    fn eval(&self, x: &Vector<'_>) -> f64 {
        let mut result = 0.0;
        for i in 0..self.centers.len() {
            let d = self.delta(x, i);
            result += self.weights[i] * d * d
                + if self.quartic {
                    0.1 * d * d * d * d
                } else {
                    0.0
                };
        }
        result
    }
    fn grad(&self, x: &Vector<'_>, g: &mut Vector<'_>) {
        for i in 0..self.centers.len() {
            let d = self.delta(x, i);
            g[i as i32 + 1] =
                2.0 * self.weights[i] * d + if self.quartic { 0.4 * d * d * d } else { 0.0 };
        }
    }
}
impl MultipleVarFunction for Bowl {
    fn variables(&self) -> i32 {
        self.centers.len() as i32
    }
    fn value(&mut self, x: &Vector<'_>, v: &mut f64) -> bool {
        *v = self.eval(x);
        self.count()
    }
}
impl MultipleVarFunctionWithGradient for Bowl {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        self.grad(x, g);
        self.count()
    }
    fn values(&mut self, x: &Vector<'_>, v: &mut f64, g: &mut Vector<'_>) -> bool {
        *v = self.eval(x);
        self.grad(x, g);
        self.count()
    }
}
impl MultipleVarFunctionWithHessian for Bowl {
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        v: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        *v = self.eval(x);
        self.grad(x, g);
        h.init(0.0);
        for i in 0..self.centers.len() {
            let d = self.delta(x, i);
            let k = i as i32 + 1;
            h[(k, k)] = 2.0 * self.weights[i] + if self.quartic { 1.2 * d * d } else { 0.0 };
        }
        self.count()
    }
}
fn newton_dump(solver: &NewtonMinimum) {
    let mut text = String::new();
    if let Err(error) = solver.dump(&mut text) {
        text.push('#');
        text.push_str(error.exception_type());
    }
    flat(&text);
    print!("{} ", solver.status() as i32);
}
pub(crate) fn minimizers_case(v: &[f64]) {
    let (n, limit, tol, cutoff) = (v[0] as usize, v[3] as i32, v[4], v[5] as i32);
    let mut f = Bowl {
        quartic: v[1] != 0.0,
        fail: v[2] as i32,
        calls: 0,
        centers: v[6..6 + n].to_vec(),
        weights: v[6 + n..6 + 2 * n].to_vec(),
    };
    let start = vector_from(&v[6 + 2 * n..6 + 3 * n]);
    gradient_minimizers(&mut f, &start, [limit, cutoff], tol);
    powell_minimizers(&mut f, &start, [limit, cutoff], tol);
    newton_minimizers(&mut f, &start, [limit, cutoff], tol);
    println!();
}
fn gradient_minimizers(f: &mut Bowl, start: &Vector<'_>, [limit, cutoff]: [i32; 2], tol: f64) {
    let n = f.variables();
    f.calls = 0;
    let mut bfgs = Bfgs::with_options(n, tol, limit, 1e-12).unwrap();
    bfgs.perform(f, start).unwrap();
    dumped(|text| bfgs.dump(text));
    f.calls = 0;
    let mut tests = 0;
    let mut bfgs = Bfgs::with_options(n, tol, limit, 1e-12).unwrap();
    let mut criterion = |_: &Bfgs, _: &mut Bowl| {
        tests += 1;
        tests >= cutoff
    };
    bfgs.perform_with_criterion(f, start, &mut criterion)
        .unwrap();
    dumped(|text| bfgs.dump(text));
    print!("{tests} ");
    f.calls = 0;
    let mut frpr = Frpr::with_options(f, tol, limit, 1e-12).unwrap();
    frpr.perform(f, start).unwrap();
    dumped(|text| frpr.dump(text));
    f.calls = 0;
    let mut tests = 0;
    let mut frpr = Frpr::with_options(f, tol, limit, 1e-12).unwrap();
    let mut criterion = |_: &Frpr, _: &mut Bowl| {
        tests += 1;
        tests >= cutoff
    };
    frpr.perform_with_criterion(f, start, &mut criterion)
        .unwrap();
    dumped(|text| frpr.dump(text));
    print!("{tests} ");
}
fn powell_minimizers(f: &mut Bowl, start: &Vector<'_>, [limit, cutoff]: [i32; 2], tol: f64) {
    let n = f.variables();
    let mut directions = Matrix::filled(1, n, 1, n, 0.0).unwrap();
    directions.set_diag(1.0).unwrap();
    f.calls = 0;
    let mut powell = Powell::with_options(f, tol, limit, 1e-12).unwrap();
    powell.perform(f, start, &directions).unwrap();
    dumped(|text| powell.dump(text));
    f.calls = 0;
    let mut tests = 0;
    let mut powell = Powell::with_options(f, tol, limit, 1e-12).unwrap();
    let mut criterion = |_: &Powell, _: &mut Bowl| {
        tests += 1;
        tests >= cutoff
    };
    powell
        .perform_with_criterion(f, start, &directions, &mut criterion)
        .unwrap();
    dumped(|text| powell.dump(text));
    print!("{tests} ");
}
fn newton_minimizers(f: &mut Bowl, start: &Vector<'_>, [limit, cutoff]: [i32; 2], tol: f64) {
    f.calls = 0;
    let mut newton = NewtonMinimum::with_options(f, tol, limit, 1e-6, true).unwrap();
    newton.perform(f, start).unwrap();
    newton_dump(&newton);
    f.calls = 0;
    let mut tests = 0;
    let mut newton = NewtonMinimum::with_options(f, tol, limit, 1e-6, true).unwrap();
    let mut criterion = |_: &NewtonMinimum| {
        tests += 1;
        tests >= cutoff
    };
    newton
        .perform_with_criterion(f, start, &mut criterion)
        .unwrap();
    newton_dump(&newton);
    print!("{tests} ");
}
