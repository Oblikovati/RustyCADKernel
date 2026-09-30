//! Direct BVH_QuickSorter_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::assertions_on_constants
)]
use rustycad_tkmath::bvh::{
    BvhQuickSorter, BvhSet, BvhSorter, BvhTriangulation, BvhVec3d, BvhVec4i, array,
};

fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!(
        (left - right).abs() <= tolerance,
        "{left} != {right} within {tolerance}"
    );
}

/// `aTriangulation.Center(index, axis)`; indices are always valid here.
fn Center(aTriangulation: &BvhTriangulation<BvhVec3d>, index: i32, axis: i32) -> f64 {
    aTriangulation.center(index, axis).unwrap()
}

include!("bvh_quick_sorter/suites_0.rs");
