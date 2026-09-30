//! Direct MathSys_Newton3D_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::field_reassign_with_default,
    clippy::needless_range_loop
)]
use rustycad_tkmath::{math_sys::*, math_utils::Status};
const THE_TOL: f64 = 1e-10;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
use rustycad_tkmath::gp::{Point3, Vector3};
include!("mathsys_newton3d/fixtures.rs");
include!("mathsys_newton3d/test_0.rs");
include!("mathsys_newton3d/test_1.rs");
include!("mathsys_newton3d/test_2.rs");
include!("mathsys_newton3d/test_3.rs");
include!("mathsys_newton3d/test_4.rs");
include!("mathsys_newton3d/test_5.rs");
include!("mathsys_newton3d/test_6.rs");
include!("mathsys_newton3d/test_7.rs");
include!("mathsys_newton3d/test_8.rs");
include!("mathsys_newton3d/test_9.rs");
