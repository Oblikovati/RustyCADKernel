//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BRepAlgoAPI_Cut_Test_1.cxx (3d097a0).
//! Tests that need BRepFilletAPI_MakeFillet (BOPTest_Utilities::CreateBlend) wait for that class.
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
pub(crate) use bop_test_utilities::*;
pub(crate) use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
pub(crate) use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Transform3, Vector3};
pub(crate) use rustycad_tkmath::toploc::Location;
pub(crate) use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
#[path = "brep_algo_api_cut_1_legacy/series_j1_j5.rs"]
mod series_j1_j5;
#[path = "brep_algo_api_cut_1_legacy/series_j6_k1.rs"]
mod series_j6_k1;
#[path = "brep_algo_api_cut_1_legacy/series_k2_k6.rs"]
mod series_k2_k6;
#[path = "brep_algo_api_cut_1_legacy/series_k7_l2.rs"]
mod series_k7_l2;
#[path = "brep_algo_api_cut_1_legacy/series_l3_m3.rs"]
mod series_l3_m3;
