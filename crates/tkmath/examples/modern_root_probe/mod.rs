//! Modern scalar root solver differential counterpart.
use rustycad_tkmath::math::{Function, FunctionWithDerivative};
use rustycad_tkmath::math_root::*;
use rustycad_tkmath::math_utils::{Config, ScalarResult};
struct Objective {
    mode: i32,
    fail: usize,
    target: f64,
    calls: Vec<(i32, f64)>,
}
impl Objective {
    fn formula(&self, x: f64, f: &mut f64, df: &mut f64) {
        match self.mode {
            0 => {
                *f = x * x - self.target;
                *df = 2.0 * x;
            }
            1 => {
                *f = x.cos() - x;
                *df = -x.sin() - 1.0;
            }
            2 => {
                *f = 1.0;
                *df = 0.0;
            }
            3 => {
                *f = x - self.target;
                *df = 1e300;
            }
            _ => {
                *f = x - self.target;
                *df = 1.0;
            }
        }
        if self.mode == 5 {
            *f = f64::NAN;
        }
        if self.mode == 6 {
            *df = f64::INFINITY;
        }
    }
}
impl Function for Objective {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        self.calls.push((0, x));
        if self.calls.len() == self.fail {
            return false;
        }
        self.formula(x, f, &mut 0.0);
        true
    }
}
impl FunctionWithDerivative for Objective {
    fn derivative(&mut self, x: f64, df: &mut f64) -> bool {
        self.formula(x, &mut 0.0, df);
        true
    }
    fn values(&mut self, x: f64, f: &mut f64, df: &mut f64) -> bool {
        self.calls.push((1, x));
        if self.calls.len() == self.fail {
            return false;
        }
        self.formula(x, f, df);
        true
    }
}
fn result(r: ScalarResult, f: &mut Objective) {
    print!("{} {} ", r.status as i32, r.iterations);
    for value in [r.root, r.value, r.derivative] {
        print!(
            "{} {} ",
            i32::from(value.is_some()),
            value.unwrap_or(91.0).to_bits()
        );
    }
    print!("{} ", f.calls.len());
    for (kind, x) in f.calls.drain(..) {
        print!("{} {} ", kind, x.to_bits());
    }
}
pub(super) fn modern_root_case(v: &[f64]) {
    let mut f: Objective = Objective {
        mode: v[0] as i32,
        fail: v[8] as usize,
        target: v[1],
        calls: Vec::new(),
    };
    let (guess, lower, upper): (f64, f64, f64) = (v[2], v[3], v[4]);
    let config: Config = Config {
        x_tolerance: v[5],
        f_tolerance: v[6],
        max_iterations: v[7] as u32,
        ..Default::default()
    };
    result(newton(&mut f, guess, &config), &mut f);
    result(newton_bounded(&mut f, guess, lower, upper, &config), &mut f);
    result(brent(&mut f, lower, upper, &config), &mut f);
    result(bisection(&mut f, lower, upper, &config), &mut f);
    result(bisection_newton(&mut f, lower, upper, &config), &mut f);
    result(secant(&mut f, lower, upper, &config), &mut f);
    result(secant_auto(&mut f, guess, &config), &mut f);
    println!();
}

pub(super) fn scalar_opt_case(v: &[f64]) {
    let mut f: Objective = Objective {
        mode: v[0] as i32,
        fail: v[8] as usize,
        target: v[1],
        calls: Vec::new(),
    };
    let (guess, lower, upper): (f64, f64, f64) = (v[2], v[3], v[4]);
    let config: Config = Config {
        x_tolerance: v[5],
        f_tolerance: v[6],
        max_iterations: v[7] as u32,
        ..Default::default()
    };
    result(
        rustycad_tkmath::math_opt::brent(&mut f, lower, upper, &config),
        &mut f,
    );
    result(
        rustycad_tkmath::math_opt::golden(&mut f, lower, upper, &config),
        &mut f,
    );
    result(
        rustycad_tkmath::math_opt::brent_with_bracket(&mut f, guess, upper - lower, &config),
        &mut f,
    );
    println!();
}
