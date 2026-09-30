//! Direct math_Crout_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{Crout, Matrix, Vector};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
include!("math_crout/suites_0.rs");
include!("math_crout/suites_1.rs");
include!("math_crout/suites_2.rs");
include!("math_crout/suites_3.rs");
