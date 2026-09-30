//! `FindAllRootsImpl` of MathRoot_MultipleUtils.hxx: sample, detect, Brent-refine.
use super::{
    MultipleIntervalHandler, MultipleRootValueCallback, MultipleSampleCallback,
    MultipleSampleLookup, add_root, effective_x_tolerance, have_opposite_signs,
    is_valid_multiple_input, sort_roots,
};
use crate::math::Function;
use crate::math_root::{MultipleConfig, MultipleResult, brent};
use crate::math_utils::{Config, ScalarResult, Status};

/// Sample count actually used: `max(2 * NbSamples, 20)` with the native size_t wrap;
/// `multiple_sample_count(100)` is 200.
pub(crate) fn multiple_sample_count(nb_samples: usize) -> usize {
    nb_samples.wrapping_mul(2).max(20)
}

/// The C++ template arguments of one instantiation, bundled for the private helpers.
struct MultipleCallbacks<'b, S, G, B, R, I> {
    sample_fn: S,
    get_value: G,
    brent_wrapper: &'b mut B,
    get_root_value: R,
    interval_extra: I,
}

/// Shared core of the multi-root search (`FindAllRootsImpl`): samples
/// `max(2 * NbSamples, 20) + 1` points, returns early when all are within NullTolerance,
/// accepts sample points within FTolerance, Brent-refines sign changes and sorts the roots.
/// `find_all_roots_impl(0.0, 1.0, &config, sample, lookup, &mut wrapper, root_value, MultipleNoExtraHandler)`.
#[allow(clippy::too_many_arguments)] // Keep the upstream eight-argument template contract.
pub fn find_all_roots_impl<S, G, B, R, I>(
    lower: f64,
    upper: f64,
    config: &MultipleConfig,
    sample_fn: S,
    get_value: G,
    brent_wrapper: &mut B,
    get_root_value: R,
    interval_extra: I,
) -> MultipleResult
where
    S: MultipleSampleCallback,
    G: MultipleSampleLookup,
    B: Function,
    R: MultipleRootValueCallback,
    I: MultipleIntervalHandler,
{
    let mut result: MultipleResult = MultipleResult::default();
    if !is_valid_multiple_input(lower, upper, config) {
        result.status = Status::InvalidInput;
        return result;
    }
    result.status = Status::Ok;
    let mut callbacks = MultipleCallbacks {
        sample_fn,
        get_value,
        brent_wrapper,
        get_root_value,
        interval_extra,
    };
    run_search(lower, upper, config, &mut callbacks, &mut result);
    result
}

fn run_search<S, G, B, R, I>(
    lower: f64,
    upper: f64,
    config: &MultipleConfig,
    callbacks: &mut MultipleCallbacks<'_, S, G, B, R, I>,
    result: &mut MultipleResult,
) where
    S: MultipleSampleCallback,
    G: MultipleSampleLookup,
    B: Function,
    R: MultipleRootValueCallback,
    I: MultipleIntervalHandler,
{
    let count: usize = multiple_sample_count(config.nb_samples);
    let eps_x: f64 = effective_x_tolerance(lower, upper, config.x_tolerance);
    let Some(xs) = sample_points(lower, upper, count, &callbacks.sample_fn, result) else {
        return;
    };
    let lookup: &G = &callbacks.get_value;
    result.is_all_null = (0..=count).all(|i| lookup.value_at(i).abs() <= config.null_tolerance);
    if result.is_all_null {
        return;
    }
    for i in 0..count {
        if !scan_interval(callbacks, config, &xs, i, eps_x, result) {
            return;
        }
    }
    if callbacks.get_value.value_at(count).abs() <= config.f_tolerance
        && !accept_root(&callbacks.get_root_value, xs[count], eps_x, result)
    {
        return;
    }
    sort_roots(result);
}

fn sample_points(
    lower: f64,
    upper: f64,
    count: usize,
    sample_fn: &impl MultipleSampleCallback,
    result: &mut MultipleResult,
) -> Option<Vec<f64>> {
    let mut xs: Vec<f64> = Vec::with_capacity(count + 1);
    for i in 0..=count {
        let ratio: f64 = i as f64 / count as f64;
        let x: f64 = (1.0 - ratio) * lower + ratio * upper;
        xs.push(x);
        if !sample_fn.sample_at(i, x) {
            result.status = sample_fn.status();
            return None;
        }
    }
    Some(xs)
}

fn scan_interval<S, G, B, R, I>(
    callbacks: &mut MultipleCallbacks<'_, S, G, B, R, I>,
    config: &MultipleConfig,
    xs: &[f64],
    i: usize,
    eps_x: f64,
    result: &mut MultipleResult,
) -> bool
where
    S: MultipleSampleCallback,
    G: MultipleSampleLookup,
    B: Function,
    R: MultipleRootValueCallback,
    I: MultipleIntervalHandler,
{
    let f0: f64 = callbacks.get_value.value_at(i);
    let f1: f64 = callbacks.get_value.value_at(i + 1);
    if f0.abs() <= config.f_tolerance {
        return accept_root(&callbacks.get_root_value, xs[i], eps_x, result);
    }
    if !have_opposite_signs(f0, f1) {
        let extra: &I = &callbacks.interval_extra;
        extra.handle_interval(i, xs[i], xs[i + 1], f0, f1, result, eps_x);
        return true;
    }
    let brent_config: Config = Config {
        x_tolerance: eps_x,
        f_tolerance: config.f_tolerance,
        max_iterations: config.max_iterations,
        ..Default::default()
    };
    let refined: ScalarResult = brent(callbacks.brent_wrapper, xs[i], xs[i + 1], &brent_config);
    result.iterations = result.iterations.wrapping_add(refined.iterations);
    match refined.root {
        Some(root) if refined.is_done() => {
            accept_root(&callbacks.get_root_value, root, eps_x, result)
        }
        _ => {
            result.status = refined.status;
            false
        }
    }
}

fn accept_root(
    get_root_value: &impl MultipleRootValueCallback,
    x: f64,
    eps_x: f64,
    result: &mut MultipleResult,
) -> bool {
    let mut value: f64 = 0.0;
    if !get_root_value.root_value(x, &mut value) {
        result.status = get_root_value.status();
        return false;
    }
    add_root(result, eps_x, x, value);
    true
}
