//! Vector-valued and tensor-product integration oracle protocol.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::{
    math::{FunctionSet, MultipleVarFunction, Vector},
    math_integ::*,
    math_utils::IntegResult,
};
struct Objective {
    mode: i32,
    fail: usize,
    equations: i32,
    calls: Vec<Vec<f64>>,
}
impl Objective {
    fn record(&mut self, x: &Vector<'_>) -> bool {
        self.calls.push(
            (0..x.length() as usize)
                .map(|i| *x.at(i).unwrap())
                .collect(),
        );
        self.calls.len() != self.fail
    }
    fn trace(&mut self) {
        print!("{} ", self.calls.len());
        for point in self.calls.drain(..) {
            print!("{} ", point.len());
            for x in point {
                print!("{} ", x.to_bits());
            }
        }
    }
}
impl MultipleVarFunction for Objective {
    fn variables(&self) -> i32 {
        0
    }
    fn value(&mut self, x: &Vector<'_>, out: &mut f64) -> bool {
        if !self.record(x) {
            return false;
        }
        if self.mode == 2 && self.calls.len().is_multiple_of(3) {
            return true;
        }
        let mut value = 1.0;
        for i in 0..x.length() as usize {
            value *= x.at(i).unwrap() + 1.0;
        }
        *out = match self.mode {
            1 => f64::NAN,
            3 => f64::MAX,
            _ => value,
        };
        true
    }
}
impl FunctionSet for Objective {
    fn variables(&self) -> i32 {
        99
    }
    fn equations(&self) -> i32 {
        self.equations
    }
    fn value(&mut self, x: &Vector<'_>, out: &mut Vector<'_>) -> bool {
        if !self.record(x) {
            return false;
        }
        if self.mode == 2 && self.calls.len().is_multiple_of(3) {
            return true;
        }
        for j in 0..out.length() as usize {
            *out.change_at(j).unwrap() = match self.mode {
                1 => f64::NAN,
                3 => f64::MAX,
                _ => x.at(0).unwrap() * x.at(0).unwrap() + j as f64,
            };
        }
        true
    }
}
fn multiple_result(r: IntegResult, f: &mut Objective) {
    print!(
        "{} {} {} {} ",
        r.status as i32,
        r.iterations,
        r.number_of_points,
        i32::from(r.value.is_some())
    );
    if let Some(value) = r.value {
        print!("{} ", value.to_bits());
    }
    f.trace();
}
fn set_result(r: SetResult, f: &mut Objective) {
    print!(
        "{} {} {} {} ",
        r.status as i32,
        r.nb_equations,
        r.nb_points,
        i32::from(r.values.is_some())
    );
    if let Some(values) = r.values {
        print!("{} ", values.length());
        for i in 0..values.length() as usize {
            print!("{} ", values.at(i).unwrap().to_bits());
        }
    }
    f.trace();
}
pub(super) fn modern_tensor_case(v: &[f64]) {
    let n = v[0] as usize;
    let mut f = Objective {
        mode: v[1] as i32,
        fail: v[2] as usize,
        equations: v[3] as i32,
        calls: Vec::new(),
    };
    let mut lower = Vector::with_bounds(-2, n as i32 - 3).unwrap();
    let mut upper = Vector::with_bounds(5, n as i32 + 4).unwrap();
    let mut orders: Array1<i32> = Array1::with_bounds(9, n as i32 + 8).unwrap();
    for i in 0..n {
        *lower.change_at(i).unwrap() = v[4] + i as f64 * 0.1;
        *upper.change_at(i).unwrap() = v[5] + i as f64 * 0.1;
        orders.as_mut_slice()[i] = v[6] as i32 + (i % 2) as i32;
    }
    let cfg = MultipleConfig {
        max_order: v[7] as usize,
    };
    multiple_result(
        gauss_multiple(&mut f, n, &lower, &upper, &orders, &cfg),
        &mut f,
    );
    multiple_result(
        gauss_multiple_uniform(&mut f, n, &lower, &upper, v[6] as usize),
        &mut f,
    );
    set_result(gauss_set(&mut f, v[4], v[5], v[6] as usize), &mut f);
    set_result(
        gauss_set_bounds(&mut f, &lower, &upper, v[6] as usize),
        &mut f,
    );
    println!();
}
