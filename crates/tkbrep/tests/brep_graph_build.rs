//! Original BRepGraph_Build_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_graph::{
        BRepGraph, EdgeId, FaceId, NodeId, NodeKind, VertexId, WireId,
        iterator::{EdgeIterator, FaceIterator, VertexIterator},
        shapes_view::{AddStatus, Options},
        tool::{edge, face, vertex},
    },
    topexp,
    topods::{self, Compound, Shape},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkprim::brep_prim_api::*;
use rustycad_tktopalgo::brep_builder_api::*;
include!("graph_build/fixtures.rs");
include!("graph_build/primitives.rs");
include!("graph_build/compounds.rs");
include!("graph_build/minimal.rs");
include!("graph_build/box_queries.rs");
include!("graph_build/flatten.rs");
include!("graph_build/natural.rs");
