//! Original BRepGraph_Fuzz_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Inputs preserve libstdc++ distributions and GCC argument evaluation; see graph_fuzz/inputs.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::{
    BRepGraph, EdgeId, FaceId, VertexId,
    editor::{EdgeOps, FaceOps, VertexOps},
    validate::{self, Options},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
include!("graph_fuzz/replay.rs");
include!("graph_fuzz/cases.rs");
