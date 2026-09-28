//! Generic system Newton result and callback trace oracle protocol.
use rustycad_tkmath::{
    math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector},
    math_sys::*,
};
struct System {
    mode: i32,
    fail: usize,
    m: usize,
    target: f64,
    scale: f64,
    calls: Vec<(i32, f64, f64)>,
}
impl System {
    fn record(&mut self, kind: i32, x: &Vector<'_>) -> bool {
        self.calls
            .push((kind, *x.at(0).unwrap(), *x.at(1).unwrap()));
        self.calls.len() != self.fail
    }
    fn residual(&self, x: &Vector<'_>, f: &mut Vector<'_>) {
        let a = *x.at(0).unwrap();
        let b = *x.at(1).unwrap();
        if self.mode == 4 && self.calls.len().is_multiple_of(3) {
            return;
        }
        for i in 0..self.m {
            *f.change_at(i).unwrap() = match self.mode {
                0 => {
                    if i == 0 {
                        a * a + b * b - self.target
                    } else {
                        a - b
                    }
                }
                2 => (i as f64 + 1.0) * (a + self.scale * b - self.target),
                3 => f64::NAN,
                5 => {
                    if i == 0 {
                        (1.0 + a) * (b - 1.0)
                    } else {
                        b * (b - 1.0)
                    }
                }
                6 => {
                    if i == 0 {
                        self.scale * (a * a - self.target)
                    } else {
                        b / self.scale
                    }
                }
                _ => {
                    if i == 0 {
                        a - self.target
                    } else if i == 1 {
                        b - 2.0 * self.target
                    } else {
                        a + b - 3.0 * self.target
                    }
                }
            };
        }
    }
    fn jacobian(&self, x: &Vector<'_>, j: &mut Matrix<'_>) {
        let a = *x.at(0).unwrap();
        let b = *x.at(1).unwrap();
        if self.mode == 4 && self.calls.len().is_multiple_of(3) {
            return;
        }
        for i in 0..self.m {
            let pair = match self.mode {
                0 => {
                    if i == 0 {
                        [2.0 * a, 2.0 * b]
                    } else {
                        [1.0, -1.0]
                    }
                }
                2 => [i as f64 + 1.0, (i as f64 + 1.0) * self.scale],
                5 => {
                    if i == 0 {
                        [b - 1.0, 1.0 + a]
                    } else {
                        [0.0, 2.0 * b - 1.0]
                    }
                }
                6 => {
                    if i == 0 {
                        [self.scale * 2.0 * a, 0.0]
                    } else {
                        [0.0, 1.0 / self.scale]
                    }
                }
                _ => {
                    if i == 0 {
                        [1.0, 0.0]
                    } else if i == 1 {
                        [0.0, 1.0]
                    } else {
                        [1.0, 1.0]
                    }
                }
            };
            *j.change_at(i, 0).unwrap() = pair[0];
            *j.change_at(i, 1).unwrap() = pair[1];
        }
    }
}
impl FunctionSet for System {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        self.m as i32
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        if !self.record(0, x) {
            return false;
        }
        self.residual(x, f);
        true
    }
}
impl FunctionSetWithDerivatives for System {
    fn derivatives(&mut self, x: &Vector<'_>, j: &mut Matrix<'_>) -> bool {
        if !self.record(2, x) {
            return false;
        }
        self.jacobian(x, j);
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, j: &mut Matrix<'_>) -> bool {
        if !self.record(1, x) {
            return false;
        }
        self.residual(x, f);
        self.jacobian(x, j);
        true
    }
}
fn vector(value: Option<Vector<'static>>) {
    print!("{} ", i32::from(value.is_some()));
    if let Some(v) = value {
        print!("{} {} ", v.lower(), v.length());
        for i in 0..v.length() as usize {
            print!("{} ", v.at(i).unwrap().to_bits());
        }
    }
}
fn result(r: SystemResult, f: &mut System) {
    print!("{} {} ", r.status as i32, r.iterations);
    vector(r.solution);
    print!("{} ", i32::from(r.value.is_some()));
    if let Some(v) = r.value {
        print!("{} ", v.to_bits());
    }
    vector(r.gradient);
    print!("{} ", i32::from(r.jacobian.is_some()));
    if let Some(j) = r.jacobian {
        print!(
            "{} {} {} {} ",
            j.lower_row(),
            j.lower_col(),
            j.row_number(),
            j.col_number()
        );
        for i in 0..j.row_number() as usize {
            for k in 0..j.col_number() as usize {
                print!("{} ", j.at(i, k).unwrap().to_bits());
            }
        }
    }
    print!(
        "{} {} {} {} {} {} ",
        r.residual_norm.to_bits(),
        r.step_norm.to_bits(),
        r.rank,
        i32::from(r.is_root),
        i32::from(r.is_stationary),
        f.calls.len()
    );
    for (kind, x, y) in f.calls.drain(..) {
        print!("{} {} {} ", kind, x.to_bits(), y.to_bits());
    }
}
pub(super) fn modern_sys_case(v: &[f64]) {
    let mut f = System {
        mode: v[0] as i32,
        m: v[1] as usize,
        fail: v[2] as usize,
        target: v[3],
        scale: v[4],
        calls: Vec::new(),
    };
    let mut start = Vector::with_bounds(3, 4).unwrap();
    start[3] = v[5];
    start[4] = v[6];
    let mut tol = Vector::with_bounds(-1, 0).unwrap();
    tol[-1] = v[7];
    tol[0] = v[8];
    let lower = Vector::filled(7, 8, v[11]).unwrap();
    let upper = Vector::filled(-3, -2, v[12]).unwrap();
    result(newton(&mut f, &start, &tol, v[9], v[10] as u32), &mut f);
    result(
        newton_bounded(&mut f, &start, &lower, &upper, &tol, v[9], v[10] as u32),
        &mut f,
    );
    result(
        newton_uniform(&mut f, &start, v[7], v[9], v[10] as u32),
        &mut f,
    );
    println!();
}

pub(super) fn modern_lm_case(v: &[f64]) {
    let mut f = System {
        mode: v[0] as i32,
        m: v[1] as usize,
        fail: v[2] as usize,
        target: v[3],
        scale: v[4],
        calls: Vec::new(),
    };
    let mut start = Vector::with_bounds(3, 4).unwrap();
    start[3] = v[5];
    start[4] = v[6];
    let lower = Vector::filled(7, 8, v[11]).unwrap();
    let upper = Vector::filled(-3, -2, v[12]).unwrap();
    let mut cfg = LMConfig::new(v[7], v[10] as u32);
    cfg.solver.x_tolerance = v[8];
    cfg.solver.f_tolerance = v[9];
    cfg.lambda_init = v[13];
    cfg.lambda_increase = v[14];
    cfg.lambda_decrease = v[15];
    cfg.lambda_max = v[16];
    cfg.lambda_min = v[17];
    cfg.rank_tolerance = v[18];
    cfg.solver.relative_tolerance = v[19];
    cfg.solver.step_min = v[20];
    result(levenberg_marquardt(&mut f, &start, &cfg), &mut f);
    result(
        levenberg_marquardt_bounded(&mut f, &start, &lower, &upper, &cfg),
        &mut f,
    );
    println!();
}
