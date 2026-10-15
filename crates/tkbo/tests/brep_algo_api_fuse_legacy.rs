//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BRepAlgoAPI_Fuse_Test.cxx (3d097a0).
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
pub(crate) use bop_test_utilities::*;
pub(crate) use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICommon, BRepAlgoAPIFuse};
pub(crate) use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Direction3, Plane3, Point3, Transform3, Vector3,
};
pub(crate) use rustycad_tkmath::toploc::Location;
pub(crate) use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
pub(crate) use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
pub(crate) use std::f64::consts::{PI, SQRT_2};
#[path = "brep_algo_api_fuse_legacy/series_a1_b3.rs"]
mod series_a1_b3;
#[path = "brep_algo_api_fuse_legacy/series_b4_c6.rs"]
mod series_b4_c6;
#[path = "brep_algo_api_fuse_legacy/series_c7_d9.rs"]
mod series_c7_d9;
#[path = "brep_algo_api_fuse_legacy/series_e1_f3.rs"]
mod series_e1_f3;
#[path = "brep_algo_api_fuse_legacy/series_f4_g6.rs"]
mod series_f4_g6;
#[path = "brep_algo_api_fuse_legacy/series_g7_h9.rs"]
mod series_g7_h9;
#[path = "brep_algo_api_fuse_legacy/series_i1_j3.rs"]
mod series_i1_j3;
#[path = "brep_algo_api_fuse_legacy/series_j4_x.rs"]
mod series_j4_x;
