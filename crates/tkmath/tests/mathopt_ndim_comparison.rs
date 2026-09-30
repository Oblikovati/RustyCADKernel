//! MathOpt_NDim_Comparison_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    clippy::field_reassign_with_default,
    clippy::default_constructed_unit_structs
)]
use rustycad_tkmath::math::{
    Bfgs, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Powell, Vector,
};
use rustycad_tkmath::math_opt::{bfgs, lbfgs, powell};
use rustycad_tkmath::math_utils::{Config, VectorResult};
const THE_TOLERANCE: f64 = 1e-6;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
fn CreateIdentityMatrix(n: i32) -> Matrix<'static> {
    let mut a = Matrix::filled(1, n, 1, n, 0.0).unwrap();
    for i in 1..=n {
        a[(i, i)] = 1.0;
    }
    a
}
include!("mathopt_ndim_comparison/fixture_0.rs");
include!("mathopt_ndim_comparison/fixture_1.rs");
include!("mathopt_ndim_comparison/fixture_2.rs");
include!("mathopt_ndim_comparison/fixture_3.rs");
include!("mathopt_ndim_comparison/fixture_4.rs");
include!("mathopt_ndim_comparison/fixture_5.rs");
include!("mathopt_ndim_comparison/fixture_6.rs");
include!("mathopt_ndim_comparison/fixture_7.rs");
include!("mathopt_ndim_comparison/fixture_8.rs");
include!("mathopt_ndim_comparison/fixture_9.rs");
include!("mathopt_ndim_comparison/fixture_10.rs");
include!("mathopt_ndim_comparison/fixture_11.rs");
include!("mathopt_ndim_comparison/tests_0.rs");
include!("mathopt_ndim_comparison/tests_1.rs");
include!("mathopt_ndim_comparison/tests_2.rs");
include!("mathopt_ndim_comparison/tests_3.rs");
