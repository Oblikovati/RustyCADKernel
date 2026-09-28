//! Multiple-root and null-interval results with complete callback traces.
use rustycad_tkmath::{
    math::{Function, FunctionWithDerivative, Vector},
    math_root::*,
};
pub(super) struct Objective {
    pub(super) mode: i32,
    pub(super) fail: usize,
    pub(super) target: f64,
    pub(super) calls: Vec<(i32, f64)>,
}
impl Objective {
    fn formula(&self, x: f64) -> (f64, f64) {
        match self.mode {
            1 => {
                let a = x - 1.003;
                let b = x - 1.007;
                (a * a * b * b, 2.0 * a * b * (a + b))
            }
            2 => (x.sin(), x.cos()),
            3 => (0.0, 0.0),
            4 => {
                if x.abs() < self.target.abs() {
                    (0.0, 0.0)
                } else {
                    (
                        x.abs() - self.target.abs(),
                        if x > 0.0 { 1.0 } else { -1.0 },
                    )
                }
            }
            5 => (f64::NAN, 0.0),
            6 => (x * x - self.target, f64::INFINITY),
            9 => (
                -(x - self.target) * (x - self.target),
                -2.0 * (x - self.target),
            ),
            _ => (x * x - self.target, 2.0 * x),
        }
    }
    fn evaluate(&mut self, event: i32, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        self.calls.push((event, x));
        if self.calls.len() == self.fail {
            return false;
        }
        let (f, df) = self.formula(x);
        if self.mode != 7 || !self.calls.len().is_multiple_of(3) {
            *value = f;
        }
        *derivative = df;
        true
    }
}
impl Function for Objective {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        self.evaluate(0, x, value, &mut 0.0)
    }
}
impl FunctionWithDerivative for Objective {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = self.formula(x).1;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        self.evaluate(1, x, value, derivative)
    }
}
pub(super) fn trace(f: &mut Objective) {
    print!("{} ", f.calls.len());
    for (kind, x) in f.calls.drain(..) {
        print!("{} {} ", kind, x.to_bits());
    }
}
pub(super) fn multiple(r: MultipleResult, f: &mut Objective) {
    print!(
        "{} {} {} {} ",
        r.status as i32,
        r.iterations,
        i32::from(r.is_all_null),
        r.nb_roots()
    );
    for i in 0..r.nb_roots() {
        print!("{} {} ", r.roots[i].to_bits(), r.values[i].to_bits());
    }
    trace(f);
}
fn all(r: AllRootsResult, f: &mut Objective) {
    print!("{} {} {} ", r.status as i32, r.nb_roots(), r.nb_intervals());
    for i in 0..r.nb_roots() {
        print!("{} {} ", r.roots[i].to_bits(), r.root_states[i]);
    }
    for interval in r.null_intervals {
        print!(
            "{} {} {} ",
            interval.a.to_bits(),
            interval.b.to_bits(),
            interval.state
        );
    }
    trace(f);
}
pub(super) fn modern_multiple_case(v: &[f64]) {
    let mut f = Objective {
        mode: v[0] as i32,
        fail: v[9] as usize,
        target: v[1],
        calls: Vec::new(),
    };
    let lower = v[2];
    let upper = v[3];
    let n = v[4] as usize;
    let cfg = MultipleConfig {
        nb_samples: n,
        x_tolerance: v[5],
        f_tolerance: v[6],
        null_tolerance: v[7],
        max_iterations: v[8] as u32,
        offset: v[10],
    };
    multiple(find_all_roots(&mut f, lower, upper, &cfg), &mut f);
    multiple(
        find_all_roots_with_derivative(&mut f, lower, upper, &cfg),
        &mut f,
    );
    all(
        find_all_roots_with_intervals(&mut f, lower, upper, n, [v[5], v[6], v[7]]),
        &mut f,
    );
    let mut samples = Vector::with_bounds(-2, 2).unwrap();
    for i in 0..5 {
        let ratio = i as f64 / 4.0;
        *samples.change_at(i).unwrap() = (1.0 - ratio) * lower + ratio * upper;
    }
    if f.mode == 10 {
        *samples.change_at(2).unwrap() = *samples.at(1).unwrap();
    }
    all(
        find_all_roots_at_samples(&mut f, &samples, [v[5], v[6], v[7]]),
        &mut f,
    );
    println!();
}
