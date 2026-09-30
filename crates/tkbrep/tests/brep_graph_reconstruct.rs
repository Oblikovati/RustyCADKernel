//! Original BRepGraph_Reconstruct_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool, EdgeCurve3D},
    brep_graph::{
        BRepGraph, EdgeId, FaceId, NodeId, NodeKind, ShellId, SolidId, TypedId, VertexId,
        editor::{EdgeOps, VertexOps},
        iterator::{DefIterator, NodeTraits},
        tool::{edge, vertex},
    },
    topexp::Explorer,
    topods::{self, Compound, Shape},
};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::{brep_builder_api::BRepBuilderAPIMakeShape, brep_gprop::BRepGProp};
use support::ref_test_tools;
include!("graph_reconstruct/fixtures.rs");
include!("graph_reconstruct/geometry.rs");
include!("graph_reconstruct/originals.rs");
include!("graph_reconstruct/mutations.rs");
