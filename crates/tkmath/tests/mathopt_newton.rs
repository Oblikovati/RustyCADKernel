//! MathOpt_Newton_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    clippy::field_reassign_with_default,
    clippy::default_constructed_unit_structs
)]
use rustycad_tkmath::math::{
    Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, MultipleVarFunctionWithHessian,
    NewtonMinimum, Vector,
};
use rustycad_tkmath::math_opt::*;
use rustycad_tkmath::math_utils::{Config, Status, VectorResult};
const THE_TOLERANCE: f64 = 1e-6;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathopt_frpr/fixture_0.rs");
include!("mathopt_frpr/fixture_1.rs");
include!("mathopt_frpr/fixture_2.rs");
include!("mathopt_frpr/fixture_3.rs");
include!("mathopt_frpr/fixture_4.rs");
include!("mathopt_newton/fixtures.rs");
include!("mathopt_newton/tests_0.rs");
include!("mathopt_newton/tests_1.rs");
include!("mathopt_newton/tests_2.rs");
include!("mathopt_newton/tests_3.rs");
include!("mathopt_newton/tests_4.rs");
