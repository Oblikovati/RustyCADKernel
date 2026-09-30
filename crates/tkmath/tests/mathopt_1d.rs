//! MathOpt_1D_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    clippy::field_reassign_with_default,
    clippy::default_constructed_unit_structs
)]
use rustycad_tkmath::math::Function;
use rustycad_tkmath::math_opt::{brent, brent_with_bracket, golden};
use rustycad_tkmath::math_utils::{Config, ScalarResult, Status};
const THE_TOLERANCE: f64 = 1e-7;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathopt_1d/fixture_0.rs");
include!("mathopt_1d/fixture_1.rs");
include!("mathopt_1d/fixture_2.rs");
include!("mathopt_1d/fixture_3.rs");
include!("mathopt_1d/fixture_4.rs");
include!("mathopt_1d/fixture_5.rs");
include!("mathopt_1d/tests_0.rs");
include!("mathopt_1d/tests_1.rs");
include!("mathopt_1d/tests_2.rs");
include!("mathopt_1d/tests_3.rs");
include!("mathopt_1d/tests_4.rs");
