//! Direct math_TrigonometricFunctionRoots_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case)]
use rustycad_tkmath::math::TrigonometricRoots;
use std::f64::consts::PI;
const TOLERANCE: f64 = 1e-6;
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
include!("math_trigonometric_function_roots/suites_0.rs");
include!("math_trigonometric_function_roots/suites_1.rs");
include!("math_trigonometric_function_roots/suites_2.rs");
include!("math_trigonometric_function_roots/suites_3.rs");
include!("math_trigonometric_function_roots/suites_4.rs");
include!("math_trigonometric_function_roots/suites_5.rs");
