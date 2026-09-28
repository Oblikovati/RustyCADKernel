//! One-variable roots, minima and integrals with default and replaced convergence tests.
use super::dumped;
use rustycad_tkmath::math::{
    BissecNewton, BracketMinimum, BracketedRoot, BrentMinimum, Function, FunctionAllRoots,
    FunctionRoot, FunctionRoots, FunctionSample, FunctionWithDerivative, GaussSingleIntegration,
    KronrodSingleIntegration, NewtonFunctionRoot,
};
/// f(x) = x^3 - c with derivative.
struct Cubic {
    c: f64,
    fail: i32,
    calls: i32,
}
impl Cubic {
    fn count(&mut self) -> bool {
        self.calls += 1;
        self.calls != self.fail
    }
}
impl Function for Cubic {
    fn value(&mut self, x: f64, v: &mut f64) -> bool {
        *v = x * x * x - self.c;
        self.count()
    }
}
impl FunctionWithDerivative for Cubic {
    fn derivative(&mut self, x: f64, d: &mut f64) -> bool {
        *d = 3.0 * x * x;
        self.count()
    }
    fn values(&mut self, x: f64, v: &mut f64, d: &mut f64) -> bool {
        *v = x * x * x - self.c;
        *d = 3.0 * x * x;
        self.count()
    }
}
/// g(x) = (x - c)^2 + s.
struct Parabola {
    c: f64,
    s: f64,
    fail: i32,
    calls: i32,
}
impl Function for Parabola {
    fn value(&mut self, x: f64, v: &mut f64) -> bool {
        *v = (x - self.c) * (x - self.c) + self.s;
        self.calls += 1;
        self.calls != self.fail
    }
}
pub(crate) fn scalar_case(v: &[f64]) {
    let (c, s, a, b, fail, cutoff) = (v[0], v[1], v[2], v[3], v[4] as i32, v[5] as i32);
    let mut cubic = Cubic { c, fail, calls: 0 };
    let mut par = Parabola {
        c,
        s,
        fail,
        calls: 0,
    };
    minimum_hooks(&mut par, [a, b], cutoff);
    root_hooks(&mut cubic, [a, b], cutoff);
    // Mirrors the native probe: later solvers run on callbacks that never fail.
    cubic.fail = 0;
    par.fail = 0;
    par.calls = 0;
    let bracket = BracketMinimum::compute(&mut par, a, b);
    dumped(|text| bracket.dump(text));
    root_dumps(&mut cubic, [a, b]);
    par.calls = 0;
    let integral = GaussSingleIntegration::new(&mut par, a, b, 5);
    dumped(|text| integral.dump(text));
    dumped(|text| GaussSingleIntegration::default().dump(text));
    print!(
        "{} ",
        i32::from(KronrodSingleIntegration::default().is_done())
    );
    println!();
}
fn minimum_hooks(par: &mut Parabola, [a, b]: [f64; 2], cutoff: i32) {
    let m = 0.5 * (a + b);
    let mut brent = BrentMinimum::new(1e-10);
    brent.perform(par, a, m, b);
    dumped(|text| brent.dump(text));
    par.calls = 0;
    let mut tests = 0;
    let mut brent = BrentMinimum::new(1e-10);
    let mut criterion = |_: &BrentMinimum, _: &mut Parabola| {
        tests += 1;
        tests >= cutoff
    };
    brent.perform_with_criterion(par, [a, m, b], &mut criterion);
    dumped(|text| brent.dump(text));
    print!("{tests} ");
}
fn root_hooks(cubic: &mut Cubic, [a, b]: [f64; 2], cutoff: i32) {
    let mut bissec = BissecNewton::new(1e-10);
    bissec.perform(cubic, a, b, 50);
    dumped(|text| bissec.dump(text));
    cubic.calls = 0;
    let mut tests = 0;
    let mut bissec = BissecNewton::new(1e-10);
    let mut criterion = |_: &BissecNewton, _: &mut Cubic| {
        tests += 1;
        tests >= cutoff
    };
    bissec.perform_with_criterion(cubic, [a, b], 50, &mut criterion);
    dumped(|text| bissec.dump(text));
    print!("{tests} ");
}
fn root_dumps(cubic: &mut Cubic, [a, b]: [f64; 2]) {
    let m = 0.5 * (a + b);
    cubic.calls = 0;
    let bracketed = BracketedRoot::new(cubic, a, b, 1e-10);
    dumped(|text| bracketed.dump(text));
    cubic.calls = 0;
    let root = FunctionRoot::new(cubic, m, 1e-10).unwrap();
    dumped(|text| root.dump(text));
    cubic.calls = 0;
    let newton = NewtonFunctionRoot::new(cubic, m, 1e-10, 1e-10);
    dumped(|text| newton.dump(text));
    cubic.calls = 0;
    let roots = FunctionRoots::new(cubic, a, b, 10);
    dumped(|text| roots.dump(text));
    cubic.calls = 0;
    let sample = FunctionSample::new(a, b, 10);
    let all = FunctionAllRoots::new(cubic, &sample, [1e-8, 1e-8, 1e-8]).unwrap();
    dumped(|text| all.dump(text));
}
