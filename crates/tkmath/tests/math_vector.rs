//! Direct math_Vector_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::{
    gp::{Xy, Xyz},
    math::{Matrix, Vector},
};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
fn check_vectors_equal(left: &Vector<'_>, right: &Vector<'_>) {
    assert_eq!(
        (left.length(), left.lower(), left.upper()),
        (right.length(), right.lower(), right.upper())
    );
    for index in left.lower()..=left.upper() {
        assert_near(left[index], right[index], 1e-7);
    }
}
include!("math_vector/suites_0.rs");
include!("math_vector/suites_1.rs");
include!("math_vector/suites_2.rs");
include!("math_vector/suites_3.rs");
include!("math_vector/suites_4.rs");
include!("math_vector/suites_5.rs");
include!("math_vector/suites_6.rs");
