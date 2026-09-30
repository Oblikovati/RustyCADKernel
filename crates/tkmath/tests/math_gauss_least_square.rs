//! Direct math_GaussLeastSquare_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{GaussLeastSquares, Matrix, Vector};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
include!("math_gauss_least_square/suites_0.rs");
include!("math_gauss_least_square/suites_1.rs");
include!("math_gauss_least_square/suites_2.rs");
include!("math_gauss_least_square/suites_3.rs");
include!("math_gauss_least_square/suites_4.rs");
