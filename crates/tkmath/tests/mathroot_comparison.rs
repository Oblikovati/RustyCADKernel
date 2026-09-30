//! Direct MathRoot_Comparison_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    clippy::field_reassign_with_default,
    clippy::default_constructed_unit_structs
)]
use rustycad_tkmath::math::{
    BissecNewton, BracketedRoot, Function, FunctionRoot, FunctionWithDerivative, NewtonFunctionRoot,
};
use rustycad_tkmath::math_root::*;
use rustycad_tkmath::math_utils::{Config, ScalarResult};
const THE_TOLERANCE: f64 = 1e-9;
fn near(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a} != {b}");
}
include!("mathroot/fixture_0.rs");
include!("mathroot/fixture_1.rs");
include!("mathroot/fixture_2.rs");
include!("mathroot/fixture_3.rs");
include!("mathroot/fixture_4.rs");
include!("mathroot_comparison/tests_0.rs");
include!("mathroot_comparison/tests_1.rs");
include!("mathroot_comparison/tests_2.rs");
