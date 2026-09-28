//! Supplemental coverage for the public MathRoot::Utils helpers (not an upstream suite).
use rustycad_tkmath::math::{Function, FunctionWithDerivative, Vector};
use rustycad_tkmath::math_root::utils::*;
use rustycad_tkmath::math_root::{MultipleConfig, MultipleResult, TrigResult, find_all_roots};
use rustycad_tkmath::math_utils::{Config, Status};
use std::cell::RefCell;
use std::f64::consts::{PI, TAU};

/// `x^2 - target` with its derivative; `failing_call` makes that 1-based call fail.
struct Parabola {
    target: f64,
    calls: usize,
    failing_call: usize,
}
impl Parabola {
    fn new(target: f64) -> Self {
        Self {
            target,
            calls: 0,
            failing_call: 0,
        }
    }
    fn counted(&mut self) -> bool {
        self.calls += 1;
        self.calls != self.failing_call
    }
}
impl Function for Parabola {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = x * x - self.target;
        self.counted()
    }
}
impl FunctionWithDerivative for Parabola {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 2.0 * x;
        self.counted()
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = x * x - self.target;
        *derivative = 2.0 * x;
        self.counted()
    }
}

#[test]
fn scalar_predicates_follow_sign_bits_and_ordering() {
    assert!(is_valid_config(&Config::default()));
    let zero_tolerance = Config {
        x_tolerance: 0.0,
        ..Default::default()
    };
    assert!(!is_valid_config(&zero_tolerance));
    assert!(is_valid_bounds(-1.0, 1.0));
    assert!(!is_valid_bounds(1.0, 1.0));
    assert!(!is_valid_bounds(0.0, f64::INFINITY));
    assert!(have_opposite_signs(-0.0, 0.0));
    assert!(!have_opposite_signs(2.0, 3.0));
    assert_eq!(midpoint(f64::MAX, f64::MAX), f64::MAX);
    assert_eq!(midpoint(-1.0, 3.0), 1.0);
}

#[test]
fn periodic_count_includes_or_excludes_the_upper_tolerance() {
    let mut count: usize = 0;
    assert!(compute_periodic_root_count(
        0.0, 6.0, 2.0, 0.0, false, &mut count
    ));
    assert_eq!(count, 4);
    assert!(compute_periodic_root_count(
        0.0, 6.0, 2.0, 0.0, true, &mut count
    ));
    assert_eq!(count, 3);
    assert!(!compute_periodic_root_count(
        0.0, 6.0, -2.0, 0.0, true, &mut count
    ));
    assert_eq!(count, 0);
}

#[test]
fn multiple_input_validation_and_effective_tolerance() {
    let config = MultipleConfig::default();
    assert!(is_valid_multiple_input(0.0, 1.0, &config));
    let one_sample = MultipleConfig {
        nb_samples: 1,
        ..config
    };
    assert!(!is_valid_multiple_input(0.0, 1.0, &one_sample));
    assert_eq!(effective_x_tolerance(0.0, 0.0, 1e-12), 1e-12);
    assert_eq!(
        effective_x_tolerance(0.0, 1.0, 1e-12),
        MULTIPLE_RELATIVE_INTERVAL_TOLERANCE
    );
    assert_eq!(effective_x_tolerance(0.0, 1.0, 1e-3), 1e-3);
    // std::max keeps the tolerance when the NaN bound makes the minimum NaN.
    assert_eq!(effective_x_tolerance(f64::NAN, 1.0, 1e-3), 1e-3);
}

#[test]
fn add_root_filters_duplicates_and_sort_roots_is_stable() {
    let mut result = MultipleResult::default();
    for (root, value) in [(3.0, 30.0), (1.0, 10.0), (1.0 + 1e-12, 11.0), (2.0, 20.0)] {
        add_root(&mut result, 1e-10, root, value);
    }
    sort_roots(&mut result);
    assert_eq!(result.roots, [1.0, 2.0, 3.0]);
    assert_eq!(result.values, [10.0, 20.0, 30.0]);
}

#[test]
fn evaluation_helpers_report_callback_and_numerical_errors() {
    let mut f = Parabola::new(4.0);
    let (mut value, mut derivative) = (0.0, 0.0);
    assert_eq!(evaluate_value(&mut f, 3.0, &mut value), Status::Ok);
    assert_eq!(value, 5.0);
    assert_eq!(
        evaluate_shifted_value(&mut f, 3.0, 1.0, &mut value),
        Status::Ok
    );
    assert_eq!(value, 4.0);
    assert_eq!(
        evaluate_shifted_values(&mut f, 3.0, 5.0, &mut value, &mut derivative),
        Status::Ok
    );
    assert_eq!((value, derivative), (0.0, 6.0));
    f.failing_call = 4;
    assert_eq!(
        evaluate_value(&mut f, 3.0, &mut value),
        Status::CallbackError
    );
    assert_eq!(
        evaluate_value(&mut f, f64::INFINITY, &mut value),
        Status::NumericalError
    );
    let huge = f64::MAX;
    assert_eq!(
        evaluate_shifted_value(&mut f, 1.0, -huge, &mut value),
        Status::Ok
    );
    assert_eq!(
        evaluate_shifted_value(&mut f, 1e154, -huge, &mut value),
        Status::NumericalError
    );
}

#[test]
fn adapters_share_the_function_and_record_status() {
    let mut f = Parabola::new(1.0);
    f.failing_call = 3;
    let shared: MultipleFunctionCell<'_, Parabola> = RefCell::new(&mut f);
    let samples: RefCell<Vector<'static>> = RefCell::new(Vector::with_size(2));
    let sample = MultipleSampleValueFn::new(&shared, &samples, 0.5);
    assert!(sample.sample_at(0, 2.0));
    assert_eq!(MultipleGetValueFn::new(&samples).value_at(0), 2.5);
    let root_value = MultipleGetRootValueFn::new(&shared);
    let mut value = 0.0;
    assert!(root_value.root_value(2.0, &mut value));
    assert_eq!((value, root_value.status()), (3.0, Status::Ok));
    assert!(!sample.sample_at(1, 1.0));
    assert_eq!(sample.status(), Status::CallbackError);
    assert!(MultipleBrentValueWrapper::new(&shared, 2.0).value(3.0, &mut value));
    assert_eq!(value, 6.0);
    assert!(MultipleDerivativeValueWrapper::new(&shared).value(3.0, &mut value));
    assert_eq!(value, 6.0);
    let mut result = MultipleResult::default();
    MultipleNoExtraHandler.handle_interval(0, 0.0, 1.0, 1.0, 2.0, &mut result, 1e-10);
    assert_eq!(result.nb_roots(), 0);
}

#[test]
fn refine_bracketed_root_polishes_with_newton() {
    let mut f = Parabola::new(2.0);
    let mut result = MultipleResult::default();
    let status = refine_bracketed_root(
        &mut f,
        0.0,
        1.0,
        -1.0,
        2.0,
        2.0,
        1e-10,
        1e-12,
        100,
        1e-10,
        &mut result,
    );
    assert_eq!(status, Status::Ok);
    assert_eq!(result.nb_roots(), 1);
    assert!((result.roots[0] - 2.0_f64.sqrt()).abs() < 1e-12);
    assert!(result.iterations > 0);
    let nan = refine_bracketed_root(
        &mut f,
        0.0,
        1.0,
        f64::NAN,
        2.0,
        2.0,
        1e-10,
        1e-12,
        100,
        1e-10,
        &mut result,
    );
    assert_eq!(nan, Status::NumericalError);
}

#[test]
fn search_entry_points_find_both_roots() {
    let config = MultipleConfig::default();
    let mut f = Parabola::new(0.25);
    let found = find_multiple_roots(&mut f, -1.0, 1.0, 10, 1e-10, 1e-10, 0.0);
    assert_eq!(found.nb_roots(), 2);
    assert!((found[0] + 0.5).abs() < 1e-10 && (found[1] - 0.5).abs() < 1e-10);
    let derivative = find_all_roots_with_derivative_impl(&mut f, -1.0, 1.0, &config);
    assert_eq!(derivative.nb_roots(), 2);
    let public = find_all_roots(&mut f, -1.0, 1.0, &config);
    let samples: RefCell<Vector<'static>> = RefCell::new(Vector::with_size(201));
    let shared = RefCell::new(&mut f);
    let mut wrapper = MultipleBrentValueWrapper::new(&shared, 0.0);
    let engine = find_all_roots_impl(
        -1.0,
        1.0,
        &config,
        MultipleSampleValueFn::new(&shared, &samples, 0.0),
        MultipleGetValueFn::new(&samples),
        &mut wrapper,
        MultipleGetRootValueFn::new(&shared),
        MultipleNoExtraHandler,
    );
    assert_eq!((engine.roots, engine.values), (public.roots, public.values));
}

#[test]
fn derivative_search_sees_a_tangential_root() {
    let mut f = Parabola::new(0.0);
    let found = find_all_roots_with_derivative_impl(&mut f, -1.0, 1.3, &MultipleConfig::default());
    assert_eq!(found.status, Status::Ok);
    assert_eq!(found.nb_roots(), 1);
    assert!(found[0].abs() < 1e-5);
}

#[test]
fn trig_root_bookkeeping_maps_and_orders_representatives() {
    assert_eq!(TRIG_REFINEMENT_MAX_ITERATIONS, 20);
    assert_eq!(
        (TRIG_REFINEMENT_STEP_TOLERANCE, TRIG_REFINEMENT_MAX_STEP),
        (1e-14, 0.5)
    );
    assert_eq!(MAX_NEWTON_POLISH_ITERATIONS, 5);
    let mut mapped = 0.0;
    assert!(map_periodic_trig_root(
        1.0 + TAU,
        0.0,
        TAU,
        1e-12,
        &mut mapped
    ));
    assert!((mapped - 1.0).abs() < 1e-14);
    assert!(!map_periodic_trig_root(4.0, 0.0, 1.0, 1e-12, &mut mapped));
    let mut result = TrigResult::default();
    for root in [3.0, 1.0, 3.0, PI, 0.5, 2.0] {
        add_distinct_trig_root(&mut result, root, 1e-9);
    }
    assert_eq!(result.nb_roots, 4);
    assert_eq!(result.roots, [0.5, 1.0, 3.0, PI]);
    let mut periodic = TrigResult::default();
    add_periodic_trig_root(&mut periodic, -PI / 2.0, 0.0, TAU, 1e-12);
    assert_eq!(periodic.nb_roots, 1);
    assert!((periodic.roots[0] - 1.5 * PI).abs() < 1e-14);
}
