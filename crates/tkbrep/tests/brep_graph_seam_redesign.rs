//! Original BRepGraph_SeamRedesign_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{
        BRepGraph, CoEdgeId, NodeId, NodeKind, WireId,
        iterator::{EdgeIterator, WireIterator},
        refs_iterator::CoEdgesOfWire,
        tool::{co_edge, edge, wire},
        validate,
    },
    topexp::Explorer,
    topods::{Shape, ShapeIterator, Vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
include!("graph_seams/fixtures.rs");
include!("graph_seams/queries.rs");
include!("graph_seams/round_trip.rs");
