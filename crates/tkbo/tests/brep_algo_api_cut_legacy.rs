//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BRepAlgoAPI_Cut_Test.cxx (3d097a0).
//! Tests that need BRepFilletAPI_MakeFillet (BOPTest_Utilities::CreateBlend) wait for that class.
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
pub(crate) use bop_test_utilities::*;
pub(crate) use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICut, BRepAlgoAPIFuse};
pub(crate) use rustycad_tkbrep::topexp::{self, Explorer};
pub(crate) use rustycad_tkbrep::topods::{self, Shape};
pub(crate) use rustycad_tkbrep::toptools::IndexedMapOfShape;
pub(crate) use rustycad_tkernel::precision;
pub(crate) use rustycad_tkg3d::top_abs::ShapeEnum;
pub(crate) use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3, Vector3};
pub(crate) use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
pub(crate) use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
pub(crate) use std::f64::consts::{PI, SQRT_2};

/// `EXPECT_NEAR(actual, expected, tolerance)`.
pub(crate) fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}
/// The number of sub-shapes of `kind` an explorer visits (`countSubShapes`).
pub(crate) fn countSubShapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

/// The number of distinct sub-shapes of `kind` (`countUniqueSubShapes`).
pub(crate) fn countUniqueSubShapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut shapes = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut shapes).unwrap();
    shapes.extent()
}

#[path = "brep_algo_api_cut_legacy/occ825.rs"]
mod occ825;
#[path = "brep_algo_api_cut_legacy/series_a1_b3.rs"]
mod series_a1_b3;
#[path = "brep_algo_api_cut_legacy/series_b4_c8.rs"]
mod series_b4_c8;
#[path = "brep_algo_api_cut_legacy/series_c9_e4.rs"]
mod series_c9_e4;
#[path = "brep_algo_api_cut_legacy/series_e5_f9.rs"]
mod series_e5_f9;
#[path = "brep_algo_api_cut_legacy/series_g1_h5.rs"]
mod series_g1_h5;
#[path = "brep_algo_api_cut_legacy/series_h6_i3.rs"]
mod series_h6_i3;
#[path = "brep_algo_api_cut_legacy/series_i4_i9.rs"]
mod series_i4_i9;
