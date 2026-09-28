//! Integration result fields and callback traces, including failure prefixes.
use rustycad_tkmath::{
    math::{Function, Vector},
    math_integ::*,
    math_utils::{IntegConfig, IntegResult, get_gauss_points_and_weights},
};
pub(super) struct Integrand {
    pub(super) mode: i32,
    pub(super) fail: usize,
    pub(super) calls: Vec<f64>,
}
impl Function for Integrand {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        self.calls.push(x);
        if self.calls.len() == self.fail {
            return false;
        }
        if self.mode == 5 && self.calls.len().is_multiple_of(3) {
            return true;
        }
        *value = match self.mode {
            1 => (30.0 * x).sin(),
            2 => 1.0 / (1.0 + x * x),
            3 => f64::NAN,
            4 => f64::MAX,
            6 => x.exp(),
            7 => -0.0,
            8 => (-x).exp(),
            9 => (-x * x).exp(),
            _ => x * x * x * x * x,
        };
        true
    }
}
fn optional(value: Option<f64>) {
    print!("{} ", i32::from(value.is_some()));
    if let Some(v) = value {
        print!("{} ", v.to_bits());
    }
}
pub(super) fn result(r: IntegResult, f: &mut Integrand) {
    print!(
        "{} {} {} ",
        r.status as i32, r.iterations, r.number_of_points
    );
    optional(r.value);
    optional(r.absolute_error);
    optional(r.relative_error);
    print!("{} ", f.calls.len());
    for x in f.calls.drain(..) {
        print!("{} ", x.to_bits());
    }
}
pub(super) fn modern_integ_case(v: &[f64]) {
    let mut f = Integrand {
        mode: v[0] as i32,
        fail: v[7] as usize,
        calls: Vec::new(),
    };
    let cfg = IntegConfig {
        initial_order: v[3] as u32,
        max_order: v[4] as u32,
        max_iterations: v[5] as u32,
        tolerance: v[6],
    };
    result(gauss(&mut f, v[1], v[2], v[3] as usize), &mut f);
    result(
        gauss_composite(&mut f, v[1], v[2], v[5] as u32, v[3] as usize),
        &mut f,
    );
    result(gauss_adaptive(&mut f, v[1], v[2], &cfg), &mut f);
    let n = v[8] as i32;
    let size = n.max(1) as usize;
    let mut points = Vector::with_size(size);
    let mut weights = Vector::with_size(size);
    let ok = get_gauss_points_and_weights(n, &mut points, &mut weights);
    print!("{} ", i32::from(ok));
    if ok {
        for i in 0..size {
            print!(
                "{} {} ",
                points.at(i).unwrap().to_bits(),
                weights.at(i).unwrap().to_bits()
            );
        }
    }
    println!();
}

pub(super) fn modern_kronrod_case(v: &[f64]) {
    let mut f = Integrand {
        mode: v[0] as i32,
        fail: v[7] as usize,
        calls: Vec::new(),
    };
    let mut cfg = KronrodConfig::new(v[6], v[5] as u32);
    cfg.nb_gauss_points = v[3] as usize;
    cfg.adaptive = v[4] != 0.0;
    result(
        kronrod_rule(&mut f, v[1], v[2], cfg.nb_gauss_points),
        &mut f,
    );
    result(kronrod(&mut f, v[1], v[2], &cfg), &mut f);
    result(
        kronrod_auto(&mut f, v[1], v[2], v[6], v[8] as usize),
        &mut f,
    );
    result(kronrod_semi_infinite(&mut f, v[1], &cfg), &mut f);
    result(kronrod_infinite(&mut f, &cfg), &mut f);
    println!();
}

pub(super) fn modern_double_exp_case(v: &[f64]) {
    let mut f = Integrand {
        mode: v[0] as i32,
        fail: v[8] as usize,
        calls: Vec::new(),
    };
    let mut cfg = DoubleExpConfig::new(v[6], v[4] as u32);
    cfg.nb_levels = v[3] as u32;
    cfg.step_factor = v[5];
    cfg.max_points_per_level = v[7] as usize;
    result(tanh_sinh(&mut f, v[1], v[2], &cfg), &mut f);
    result(exp_sinh(&mut f, v[1], &cfg), &mut f);
    result(sinh_sinh(&mut f, &cfg), &mut f);
    let bounds = match v[9] as i32 {
        1 => [v[1], f64::INFINITY],
        2 => [f64::NEG_INFINITY, v[2]],
        3 => [f64::NEG_INFINITY, f64::INFINITY],
        4 => [f64::INFINITY, v[2]],
        5 => [f64::NAN, v[2]],
        _ => [v[1], v[2]],
    };
    result(
        double_exponential(&mut f, bounds[0], bounds[1], &cfg),
        &mut f,
    );
    result(
        tanh_sinh_with_singularity(&mut f, v[1], v[2], v[10], &cfg),
        &mut f,
    );
    println!();
}
