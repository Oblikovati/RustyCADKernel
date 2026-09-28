//! Refinement and search-engine entry points of MathRoot::Utils.
use super::{adapters, evaluations, multiple, trace};
use crate::modern_multiple_probe::Objective;
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::math_root::{MultipleConfig, MultipleResult, utils};
use std::cell::RefCell;

fn config(v: &[f64]) -> MultipleConfig {
    MultipleConfig {
        nb_samples: v[4] as usize,
        x_tolerance: v[5],
        f_tolerance: v[6],
        null_tolerance: v[7],
        max_iterations: v[8] as u32,
        offset: v[10],
    }
}

fn shifted_bracket(f: &mut Objective, lower: f64, upper: f64, offset: f64) -> (f64, f64) {
    let (mut y1, mut y2): (f64, f64) = (0.0, 0.0);
    utils::evaluate_shifted_value(f, lower, offset, &mut y1);
    utils::evaluate_shifted_value(f, upper, offset, &mut y2);
    trace(f);
    (y1, y2)
}

fn refined(f: &mut Objective, lower: f64, upper: f64, cfg: &MultipleConfig) {
    let (y1, y2) = shifted_bracket(f, lower, upper, cfg.offset);
    let mut result: MultipleResult = MultipleResult::default();
    let status = utils::refine_bracketed_root(
        f,
        cfg.offset,
        lower,
        y1,
        upper,
        y2,
        cfg.x_tolerance,
        cfg.f_tolerance,
        cfg.max_iterations,
        cfg.x_tolerance,
        &mut result,
    );
    print!("{} ", status as i32);
    multiple(result, f);
}

fn engine(f: &mut Objective, lower: f64, upper: f64, cfg: &MultipleConfig) -> MultipleResult {
    let count: usize = cfg.nb_samples.wrapping_mul(2).max(20);
    let samples: RefCell<Vector<'static>> = RefCell::new(Vector::with_size(count + 1));
    let shared = RefCell::new(f);
    let mut wrapper = utils::MultipleBrentValueWrapper::new(&shared, cfg.offset);
    utils::find_all_roots_impl(
        lower,
        upper,
        cfg,
        utils::MultipleSampleValueFn::new(&shared, &samples, cfg.offset),
        utils::MultipleGetValueFn::new(&samples),
        &mut wrapper,
        utils::MultipleGetRootValueFn::new(&shared),
        utils::MultipleNoExtraHandler,
    )
}

/// `mathroot_search` with the modern_multiple request layout: evaluations, adapters,
/// RefineBracketedRoot, FindMultipleRoots, FindAllRootsImpl and FindAllRootsWithDerivativeImpl.
pub(crate) fn mathroot_search_case(v: &[f64]) {
    let mut f = Objective {
        mode: v[0] as i32,
        fail: v[9] as usize,
        target: v[1],
        calls: Vec::new(),
    };
    let (lower, upper): (f64, f64) = (v[2], v[3]);
    let cfg: MultipleConfig = config(v);
    evaluations(&mut f, lower, cfg.offset);
    adapters(&mut f, upper, cfg.offset);
    refined(&mut f, lower, upper, &cfg);
    let found = utils::find_multiple_roots(
        &mut f,
        lower,
        upper,
        cfg.nb_samples,
        cfg.x_tolerance,
        cfg.f_tolerance,
        cfg.offset,
    );
    multiple(found, &mut f);
    let found = engine(&mut f, lower, upper, &cfg);
    multiple(found, &mut f);
    let found = utils::find_all_roots_with_derivative_impl(&mut f, lower, upper, &cfg);
    multiple(found, &mut f);
    println!();
}
