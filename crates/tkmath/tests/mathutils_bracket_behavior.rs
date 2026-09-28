//! Supplemental edge cases for MathUtils_Bracket.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math_utils::*;

#[test]
fn root_expansion_and_reversed_endpoints() {
    let mut linear = |x: f64, out: &mut f64| {
        *out = x - 10.0;
        true
    };
    let expanded = bracket_root(&mut linear, 0.0, 1.0, 50);
    assert_eq!(expanded.status, Status::Ok);
    assert!(expanded.a < 10.0 && expanded.b > 10.0);
    assert!(expanded.iterations > 0);
    let reversed = bracket_root(&mut linear, 11.0, 9.0, 50);
    assert_eq!(
        (reversed.a, reversed.b, reversed.fa, reversed.fb),
        (9.0, 11.0, -1.0, 1.0)
    );
}
#[test]
fn root_partial_failures_and_limits() {
    let mut failing = |x: f64, out: &mut f64| {
        *out = x;
        x == 0.0
    };
    let partial = bracket_root(&mut failing, 0.0, 1.0, 50);
    assert_eq!(partial.status, Status::CallbackError);
    assert!(partial.has_fa && !partial.has_fb);
    assert_eq!(partial.fb, 1.0);
    assert_eq!(
        bracket_root(&mut failing, 0.0, 0.0, 50).status,
        Status::InvalidInput
    );
    let mut constant = |_: f64, out: &mut f64| {
        *out = 1.0;
        true
    };
    let exhausted = bracket_root(&mut constant, 0.0, 1.0, 2);
    assert_eq!(
        (exhausted.status, exhausted.iterations),
        (Status::MaxIterations, 2)
    );
    assert_eq!(
        bracket_root(&mut constant, -f64::MAX, f64::MAX, 2).status,
        Status::NumericalError
    );
}
#[test]
fn minimum_cached_endpoints_and_bounds() {
    let mut quadratic = |x: f64, out: &mut f64| {
        *out = (x - 2.0) * (x - 2.0);
        x != 0.0 && x != 1.0
    };
    let options = MinBracketOptions {
        has_fa: true,
        has_fb: true,
        fa: 4.0,
        fb: 1.0,
        ..Default::default()
    };
    let result = bracket_minimum(&mut quadratic, 0.0, 1.0, &options);
    assert!(result.is_valid);
    assert!(result.a < result.b && result.b < result.c);
    assert!(result.fb < result.fa && result.fb < result.fc);
    let restrictive = MinBracketOptions {
        use_limits: true,
        left_limit: 0.0,
        right_limit: 1.0,
        ..options
    };
    assert_eq!(
        bracket_minimum(&mut quadratic, 0.0, 1.0, &restrictive).status,
        Status::NoSolution
    );
}
#[test]
fn minimum_failures_and_reversed_search() {
    let mut quadratic = |x: f64, out: &mut f64| {
        *out = (x - 2.0) * (x - 2.0);
        true
    };
    let options = MinBracketOptions::default();
    assert!(bracket_minimum(&mut quadratic, 4.0, 3.0, &options).is_valid);
    assert_eq!(
        bracket_minimum(&mut quadratic, 1.0, 1.0, &options).status,
        Status::InvalidInput
    );
    let mut nonfinite = |_: f64, out: &mut f64| {
        *out = f64::NAN;
        true
    };
    assert_eq!(
        bracket_minimum(&mut nonfinite, 0.0, 1.0, &options).status,
        Status::NumericalError
    );
    let mut constant = |_: f64, out: &mut f64| {
        *out = 1.0;
        true
    };
    let exhausted = bracket_minimum(&mut constant, 0.0, 1.0, &options);
    assert_eq!(
        (exhausted.status, exhausted.iterations),
        (Status::MaxIterations, 50)
    );
}
