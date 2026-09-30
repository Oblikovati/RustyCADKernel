//! Direct math_DirectPolynomialRoots_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, clippy::excessive_precision)]
use rustycad_tkmath::math::DirectPolynomialRoots;
const MY_TOLERANCE: f64 = 1e-10;
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
fn verify_root(root: f64, coefficients: &[f64], index: i32, tolerance: f64) {
    let mut result: f64 = 0.0;
    let mut power: f64 = 1.0;
    for coefficient in coefficients.iter().rev() {
        result += coefficient * power;
        power *= root;
    }
    assert!(
        result.abs() <= tolerance,
        "Root {index} should satisfy polynomial equation: {result}"
    );
}
include!("math_direct_polynomial_roots/suites_0.rs");
include!("math_direct_polynomial_roots/suites_1.rs");
include!("math_direct_polynomial_roots/suites_2.rs");
include!("math_direct_polynomial_roots/suites_3.rs");
include!("math_direct_polynomial_roots/suites_4.rs");
include!("math_direct_polynomial_roots/suites_5.rs");
include!("math_direct_polynomial_roots/suites_6.rs");
