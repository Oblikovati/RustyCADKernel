//! MathRoot::Utils public helpers; twin of tools/oracle/tkmath_mathroot_api_probe.cxx.
use super::modern_multiple_probe::{Objective, multiple, trace};
use rustycad_tkmath::math::{Function, Vector};
use rustycad_tkmath::math_root::{MultipleConfig, MultipleResult, TrigResult, utils};
use rustycad_tkmath::math_utils::Config;
use std::cell::RefCell;
mod search;
pub(super) use search::mathroot_search_case;

fn flag(value: bool) -> i32 {
    i32::from(value)
}

/// `mathroot_utils a b c d flag n`: scalar predicates, periodic counts and trig mapping.
pub(super) fn mathroot_utils_case(v: &[f64]) {
    let (a, b, c, d) = (v[0], v[1], v[2], v[3]);
    let n: usize = v[5] as usize;
    print!(
        "{} {} {} ",
        flag(utils::is_valid_bounds(a, b)),
        flag(utils::have_opposite_signs(a, b)),
        utils::midpoint(a, b).to_bits()
    );
    let config = Config {
        x_tolerance: c,
        f_tolerance: d,
        max_iterations: n as u32,
        ..Default::default()
    };
    print!("{} ", flag(utils::is_valid_config(&config)));
    let mut count: usize = 77;
    let counted: bool = utils::compute_periodic_root_count(a, b, c, d, v[4] != 0.0, &mut count);
    print!(
        "{} {} {} ",
        flag(counted),
        count,
        utils::effective_x_tolerance(a, b, c).to_bits()
    );
    let multiple = MultipleConfig {
        nb_samples: n,
        x_tolerance: c,
        f_tolerance: d,
        null_tolerance: d,
        max_iterations: n as u32,
        offset: a,
    };
    print!("{} ", flag(utils::is_valid_multiple_input(a, b, &multiple)));
    let mut mapped: f64 = 91.0;
    let is_mapped: bool = utils::map_periodic_trig_root(a, b, c, d, &mut mapped);
    println!("{} {}", flag(is_mapped), mapped.to_bits());
}

/// `mathroot_trig_roots lower upper tolerance k r...`: alternating periodic/distinct inserts.
pub(super) fn mathroot_trig_roots_case(v: &[f64]) {
    let (lower, upper, tolerance) = (v[0], v[1], v[2]);
    let mut result: TrigResult = TrigResult::default();
    for (i, &root) in v[4..4 + v[3] as usize].iter().enumerate() {
        if i % 2 == 0 {
            utils::add_periodic_trig_root(&mut result, root, lower, upper, tolerance);
        } else {
            utils::add_distinct_trig_root(&mut result, root, tolerance);
        }
    }
    print!("{} ", result.nb_roots);
    for root in &result.roots[..result.nb_roots] {
        print!("{} ", root.to_bits());
    }
    println!();
}

/// `mathroot_sort eps k (root value)...`: duplicate filtering then insertion sort.
pub(super) fn mathroot_sort_case(v: &[f64]) {
    let mut result: MultipleResult = MultipleResult::default();
    for pair in v[2..2 + 2 * v[1] as usize].chunks(2) {
        utils::add_root(&mut result, v[0], pair[0], pair[1]);
    }
    utils::sort_roots(&mut result);
    print!("{} ", result.nb_roots());
    for (root, value) in result.roots.iter().zip(&result.values) {
        print!("{} {} ", root.to_bits(), value.to_bits());
    }
    println!();
}

fn evaluations(f: &mut Objective, x: f64, offset: f64) {
    let (mut value, mut derivative): (f64, f64) = (91.0, 92.0);
    let plain = utils::evaluate_value(f, x, &mut value) as i32;
    print!("{} {} ", plain, value.to_bits());
    let shifted = utils::evaluate_shifted_value(f, x, offset, &mut value) as i32;
    print!("{} {} ", shifted, value.to_bits());
    let both = utils::evaluate_shifted_values(f, x, offset, &mut value, &mut derivative) as i32;
    print!("{} {} {} ", both, value.to_bits(), derivative.to_bits());
    trace(f);
}

fn sample_adapters(shared: &utils::MultipleFunctionCell<'_, Objective>, x: f64, offset: f64) {
    use utils::{MultipleRootValueCallback, MultipleSampleCallback, MultipleSampleLookup};
    let samples: RefCell<Vector<'static>> = RefCell::new(Vector::with_size(3));
    samples.borrow_mut().init(93.0);
    let sample = utils::MultipleSampleValueFn::new(shared, &samples, offset);
    let sampled: bool = sample.sample_at(1, x);
    let lookup = utils::MultipleGetValueFn::new(&samples);
    print!(
        "{} {} {} ",
        flag(sampled),
        sample.status() as i32,
        lookup.value_at(1).to_bits()
    );
    let root_value = utils::MultipleGetRootValueFn::new(shared);
    let mut value: f64 = 94.0;
    let valued: bool = root_value.root_value(x, &mut value);
    print!(
        "{} {} {} ",
        flag(valued),
        root_value.status() as i32,
        value.to_bits()
    );
}

fn value_adapters(shared: &utils::MultipleFunctionCell<'_, Objective>, x: f64, offset: f64) {
    let mut value: f64 = 95.0;
    let shifted: bool = utils::MultipleBrentValueWrapper::new(shared, offset).value(x, &mut value);
    print!("{} {} ", flag(shifted), value.to_bits());
    value = 96.0;
    let slope: bool = utils::MultipleDerivativeValueWrapper::new(shared).value(x, &mut value);
    print!("{} {} ", flag(slope), value.to_bits());
}

fn adapters(f: &mut Objective, x: f64, offset: f64) {
    let shared = RefCell::new(&mut *f);
    sample_adapters(&shared, x, offset);
    value_adapters(&shared, x, offset);
    trace(f);
}
