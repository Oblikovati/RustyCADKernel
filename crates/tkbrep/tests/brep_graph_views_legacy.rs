//! Original BRepGraph_Views_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{
        defs_iterator::DefsVertexOfEdge,
        layer_history::LayerHistory,
        layer_topo_supplement::LayerTopoSupplement,
        refs_iterator::RefsVertexOfEdge,
        related_iterator::{RelatedIterator, RelationKind},
        tool::{co_edge, edge, face, vertex},
        *,
    },
    topods::{Compound, Edge, Face, Shape, Shell, Vertex, Wire},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
include!("graph_views/fixtures.rs");
include!("graph_views/topology.rs");
include!("graph_views/uids.rs");
include!("graph_views/refs.rs");
include!("graph_views/shapes.rs");
include!("graph_views/cache.rs");
include!("graph_views/editor.rs");
include!("graph_views/regressions.rs");
