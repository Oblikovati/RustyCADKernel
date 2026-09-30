//! Direct MathOpt_PSO_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::field_reassign_with_default
)]
use rustycad_tkmath::{
    math::{MultipleVarFunction, Pso, Vector},
    math_opt::*,
    math_utils::{Status, VectorResult},
};
const THE_TOLERANCE: f64 = 1e-6;
const THE_STOCHASTIC_TOLERANCE: f64 = 1e-3;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathopt_pso/fixture_SphereFunc.rs");
include!("mathopt_pso/fixture_RastriginFunc.rs");
include!("mathopt_pso/fixture_AckleyFunc.rs");
include!("mathopt_pso/fixture_RosenbrockFunc.rs");
include!("mathopt_pso/fixture_BoothFunc.rs");
include!("mathopt_pso/fixture_PartiallyDefinedFunc.rs");
include!("mathopt_pso/fixture_RejectAllFunc.rs");
include!("mathopt_pso/fixture_SphereFuncOld.rs");
include!("mathopt_pso/test_0.rs");
include!("mathopt_pso/test_1.rs");
include!("mathopt_pso/test_2.rs");
include!("mathopt_pso/test_3.rs");
include!("mathopt_pso/test_4.rs");
include!("mathopt_pso/test_5.rs");
include!("mathopt_pso/test_6.rs");
include!("mathopt_pso/test_7.rs");
include!("mathopt_pso/test_8.rs");
include!("mathopt_pso/test_9.rs");
include!("mathopt_pso/test_10.rs");
include!("mathopt_pso/test_11.rs");
include!("mathopt_pso/test_12.rs");
include!("mathopt_pso/test_13.rs");
include!("mathopt_pso/test_14.rs");
include!("mathopt_pso/test_15.rs");
include!("mathopt_pso/test_16.rs");
include!("mathopt_pso/test_17.rs");
include!("mathopt_pso/test_18.rs");
include!("mathopt_pso/test_19.rs");
include!("mathopt_pso/test_20.rs");
include!("mathopt_pso/test_21.rs");
include!("mathopt_pso/test_22.rs");
include!("mathopt_pso/test_23.rs");
include!("mathopt_pso/test_24.rs");
include!("mathopt_pso/test_25.rs");
include!("mathopt_pso/test_26.rs");
include!("mathopt_pso/test_27.rs");
