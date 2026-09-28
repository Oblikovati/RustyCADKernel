//! Differential coverage for modern optimization callback and result semantics.
use rustycad_tkmath::math::{Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use rustycad_tkmath::math_opt::*;
use rustycad_tkmath::math_utils::{Config, VectorResult};
struct Objective {
    mode: i32,
    fail: usize,
    center: f64,
    calls: Vec<f64>,
}
impl Objective {
    fn record(&mut self, kind: f64, x: &Vector<'_>) -> bool {
        self.calls
            .extend([kind, *x.at(0).unwrap(), *x.at(1).unwrap()]);
        self.calls.len() / 3 != self.fail
    }
}
impl MultipleVarFunction for Objective {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        if !self.record(0.0, x) {
            return false;
        }
        let a: f64 = *x.at(0).unwrap() - self.center;
        let b: f64 = *x.at(1).unwrap() + self.center;
        *f = match self.mode {
            1 => 1.0,
            2 => *x.at(0).unwrap(),
            3 => f64::INFINITY,
            _ => a * a + 3.0 * b * b,
        };
        true
    }
}
impl MultipleVarFunctionWithGradient for Objective {
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }

    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        if !self.record(1.0, x) {
            return false;
        }
        *g.change_at(0).unwrap() = 2.0 * (*x.at(0).unwrap() - self.center);
        *g.change_at(1).unwrap() = 6.0 * (*x.at(1).unwrap() + self.center);
        if self.mode == 1 {
            g.init(0.0);
        }
        if self.mode == 2 {
            *g.change_at(0).unwrap() = 1.0;
            *g.change_at(1).unwrap() = 0.0;
        }
        if self.mode == 4 {
            *g.change_at(1).unwrap() = f64::INFINITY;
        }
        true
    }
}
fn result(r: VectorResult, f: &mut Objective) {
    print!(
        "{} {} {} {} ",
        r.status as i32,
        r.iterations,
        i32::from(r.value.is_some()),
        r.value.unwrap_or(91.0).to_bits()
    );
    for v in [&r.solution, &r.gradient] {
        print!("{} ", i32::from(v.is_some()));
        if let Some(v) = v {
            for i in 0..v.length() {
                print!("{} ", v.at(i as usize).unwrap().to_bits());
            }
        }
    }
    print!("{} ", f.calls.len());
    for c in f.calls.drain(..) {
        print!("{} ", c.to_bits());
    }
}
pub(super) fn modern_opt_case(v: &[f64]) {
    let mut f: Objective = Objective {
        mode: v[0] as i32,
        fail: v[1] as usize,
        center: v[3],
        calls: Vec::new(),
    };
    let mut start: Vector<'_> = Vector::with_bounds(-2, -1).unwrap();
    start[-2] = v[4];
    start[-1] = v[5];
    let cfg: Config = Config {
        max_iterations: v[2] as u32,
        x_tolerance: v[6],
        f_tolerance: v[6],
        ..Default::default()
    };
    let mut directions: Matrix<'_> = Matrix::with_bounds(-1, 0, 3, 4).unwrap();
    directions[(-1, 3)] = 1.0;
    directions[(-1, 4)] = 0.5;
    directions[(0, 3)] = -0.5;
    directions[(0, 4)] = 1.0;
    result(powell(&mut f, &start, &cfg), &mut f);
    result(
        powell_with_directions(&mut f, &start, &directions, &cfg),
        &mut f,
    );
    result(bfgs(&mut f, &start, &cfg), &mut f);
    result(bfgs_numerical(&mut f, &start, v[7], &cfg), &mut f);
    println!();
}

pub(super) fn frpr_case(v: &[f64]) {
    let mut f: Objective = Objective {
        mode: v[0] as i32,
        fail: v[1] as usize,
        center: v[3],
        calls: Vec::new(),
    };
    let mut start: Vector<'_> = Vector::with_bounds(-2, -1).unwrap();
    start[-2] = v[4];
    start[-1] = v[5];
    let cfg: Config = Config {
        max_iterations: v[2] as u32,
        x_tolerance: v[6],
        f_tolerance: v[6],
        ..Default::default()
    };
    for formula in [
        ConjugateGradientFormula::FletcherReeves,
        ConjugateGradientFormula::PolakRibiere,
        ConjugateGradientFormula::HestenesStiefel,
        ConjugateGradientFormula::DaiYuan,
        ConjugateGradientFormula::Invalid(4),
    ] {
        let options: FrprConfig = FrprConfig {
            base: cfg,
            formula,
            restart_interval: v[0] as u32,
        };
        result(frpr(&mut f, &start, &options), &mut f);
        result(frpr_numerical(&mut f, &start, v[7], &options), &mut f);
    }
    println!();
}

impl HessianFunction for Objective {
    fn hessian(&mut self, x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        if !self.record(2.0, x) {
            return false;
        }
        *h.change_at(0, 0).unwrap() = if self.mode == 5 { -2.0 } else { 2.0 };
        *h.change_at(1, 1).unwrap() = 6.0;
        *h.change_at(0, 1).unwrap() = if self.mode == 6 { 1.0 } else { 0.0 };
        *h.change_at(1, 0).unwrap() = 0.0;
        if self.mode == 7 {
            *h.change_at(0, 0).unwrap() = f64::INFINITY;
        }
        true
    }
}
pub(super) fn modern_newton_case(v: &[f64]) {
    let mode: i32 = v[0] as i32;
    let mut f: Objective = Objective {
        mode,
        fail: v[1] as usize,
        center: v[3],
        calls: Vec::new(),
    };
    let mut start: Vector<'_> = Vector::with_bounds(-2, -1).unwrap();
    start[-2] = v[4];
    start[-1] = v[5];
    let cfg: NewtonConfig = NewtonConfig {
        base: Config {
            max_iterations: v[2] as u32,
            x_tolerance: v[6],
            f_tolerance: v[6],
            ..Default::default()
        },
        use_line_search: mode % 2 == 0,
        regularization: if mode == 8 { 0.0 } else { 1e-8 },
    };
    result(newton(&mut f, &start, &cfg), &mut f);
    result(newton_modified(&mut f, &start, &cfg), &mut f);
    result(newton_numerical_hessian(&mut f, &start, v[7], &cfg), &mut f);
    result(newton_numerical(&mut f, &start, 1e-8, v[7], &cfg), &mut f);
    let lo: Vector<'_> = Vector::filled(-1, 0, -1.0).unwrap();
    let mut hi: Vector<'_> = Vector::filled(3, 4, 1.0).unwrap();
    if mode == 9 {
        hi[3] = -1.0;
    }
    result(newton_bounded(&mut f, &start, &lo, &hi, &cfg), &mut f);
    result(bfgs_bounded(&mut f, &start, &lo, &hi, &cfg.base), &mut f);
    println!();
}

pub(super) fn modern_lbfgs_case(v: &[f64]) {
    let mut f: Objective = Objective {
        mode: v[0] as i32,
        fail: v[1] as usize,
        center: v[3],
        calls: Vec::new(),
    };
    let mut start: Vector<'_> = Vector::with_bounds(-2, -1).unwrap();
    start[-2] = v[4];
    start[-1] = v[5];
    let cfg: Config = Config {
        max_iterations: v[2] as u32,
        x_tolerance: v[6],
        f_tolerance: v[6],
        ..Default::default()
    };
    for memory in [0, 1, 2, 10, usize::MAX] {
        result(lbfgs(&mut f, &start, memory, &cfg), &mut f);
    }
    println!();
}
