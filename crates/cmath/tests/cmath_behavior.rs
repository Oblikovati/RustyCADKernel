//! The adapter must reach libm and keep its special-value contract.
use rustycad_cmath::{acosh, asinh, atanh, remainder};

#[test]
fn special_values_follow_c99() {
    assert!(asinh(-0.0).is_sign_negative() && asinh(f64::INFINITY) == f64::INFINITY);
    assert!(acosh(0.5).is_nan() && acosh(f64::INFINITY) == f64::INFINITY);
    assert_eq!(atanh(1.0), f64::INFINITY);
    assert!(atanh(2.0).is_nan() && asinh(f64::NAN).is_nan());
}

#[test]
fn values_are_inverse_hyperbolic_functions() {
    for x in [1.0e-300, 1.0e-9, 0.5, 2.0, 30.0, 1.0e300] {
        assert!((asinh(x).sinh() - x).abs() <= 1.0e-12 * x);
    }
    assert!((acosh(2.0).cosh() - 2.0).abs() <= 1.0e-15);
    assert!((atanh(0.5).tanh() - 0.5).abs() <= 1.0e-15);
}

#[test]
fn remainder_rounds_the_quotient_to_nearest_even() {
    assert_eq!(remainder(5.0, 2.0), 1.0);
    assert_eq!(remainder(7.0, 2.0), -1.0);
    assert_eq!(remainder(-0.0, 1.0).to_bits(), (-0.0f64).to_bits());
    assert!(remainder(1.0, 0.0).is_nan() && remainder(f64::INFINITY, 1.0).is_nan());
    assert_eq!(
        remainder(2.0 * std::f64::consts::PI, std::f64::consts::PI),
        0.0
    );
}
