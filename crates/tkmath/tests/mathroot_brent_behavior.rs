//! Supplemental MathRoot_Brent.hxx cases at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math_root::brent;
use rustycad_tkmath::math_utils::{Config, Status};
#[test]
fn brent_irrational_root_and_endpoint_convergence() {
    let mut square = |x: f64, out: &mut f64| {
        *out = x * x - 2.0;
        true
    };
    let result = brent(&mut square, 0.0, 2.0, &Config::default());
    assert_eq!(result.status, Status::Ok);
    assert!((result.root.unwrap() - 2.0_f64.sqrt()).abs() < 1e-10);
    let result = brent(
        &mut |x: f64, out: &mut f64| {
            *out = x - 2.0;
            true
        },
        2.0,
        3.0,
        &Config::default(),
    );
    assert_eq!(result.status, Status::Ok);
    assert_eq!(result.iterations, 0);
    assert_eq!(result.root, Some(2.0));
}
#[test]
fn brent_rejects_reversed_brackets_and_callback_failure() {
    let mut square = |x: f64, out: &mut f64| {
        *out = x * x - 2.0;
        true
    };
    assert_eq!(
        brent(&mut square, 2.0, 0.0, &Config::default()).status,
        Status::InvalidInput
    );
    let mut calls: usize = 0;
    let mut failing = |x: f64, out: &mut f64| {
        calls += 1;
        *out = x * x - 2.0;
        calls != 3
    };
    let result = brent(&mut failing, 0.0, 2.0, &Config::default());
    assert_eq!(result.status, Status::CallbackError);
    assert_eq!(result.iterations, 1);
    assert!(result.root.is_none());
}
#[test]
fn brent_exhaustion_keeps_last_evaluated_point() {
    let config = Config {
        max_iterations: 1,
        ..Default::default()
    };
    let result = brent(
        &mut |x: f64, out: &mut f64| {
            *out = x * x - 2.0;
            true
        },
        0.0,
        2.0,
        &config,
    );
    assert_eq!(result.status, Status::MaxIterations);
    assert_eq!(
        result.value,
        Some(result.root.unwrap() * result.root.unwrap() - 2.0)
    );
    let invalid = brent(
        &mut |_: f64, out: &mut f64| {
            *out = f64::NAN;
            true
        },
        0.0,
        2.0,
        &config,
    );
    assert_eq!(invalid.status, Status::NumericalError);
    assert!(invalid.root.is_none());
}

#[test]
fn secant_initialization_and_stopping() {
    use rustycad_tkmath::math_root::{secant, secant_auto};
    let mut square = |x: f64, out: &mut f64| {
        *out = x * x - 2.0;
        true
    };
    let result = secant(&mut square, 1.0, 2.0, &Config::default());
    assert_eq!(result.status, Status::Ok);
    assert!((result.root.unwrap() - 2.0_f64.sqrt()).abs() < 1e-10);
    assert_eq!(
        secant_auto(&mut square, 1.0, &Config::default()).status,
        Status::Ok
    );
    assert_eq!(
        secant(&mut square, 1.0, 1.0, &Config::default()).status,
        Status::InvalidInput
    );
    assert_eq!(
        secant_auto(&mut square, f64::MAX, &Config::default()).status,
        Status::InvalidInput
    );
}
#[test]
fn secant_small_step_requires_residual_and_failures_keep_previous_sample() {
    use rustycad_tkmath::math_root::secant;
    let config = Config {
        x_tolerance: 10.0,
        f_tolerance: 0.0,
        ..Default::default()
    };
    let result = secant(
        &mut |x: f64, out: &mut f64| {
            *out = x * x - 2.0;
            true
        },
        1.0,
        2.0,
        &config,
    );
    assert_eq!(result.status, Status::NotConverged);
    let mut count: usize = 0;
    let mut failing = |x: f64, out: &mut f64| {
        count += 1;
        *out = x * x - 2.0;
        count != 3
    };
    let result = secant(&mut failing, 1.0, 2.0, &config);
    assert_eq!(result.status, Status::CallbackError);
    assert_eq!(result.root, Some(2.0));
    assert_eq!(result.value, Some(2.0));
}
