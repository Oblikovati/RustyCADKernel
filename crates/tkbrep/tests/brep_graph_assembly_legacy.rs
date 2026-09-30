//! Original BRepGraph_Assembly_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{
        editor::OccurrenceOps,
        iterator::{DefIterator, RootProductIterator},
        shapes_view::Options,
        *,
    },
    topexp::Explorer,
    topods::{Compound, Shape, ShapeIterator},
};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
use rustycad_tktopalgo::{brep_bnd_lib::BRepBndLib, brep_builder_api::BRepBuilderAPIMakeShape};
include!("graph_assembly/fixtures.rs");
include!("graph_assembly/basics.rs");
include!("graph_assembly/links.rs");
include!("graph_assembly/roots.rs");
include!("graph_assembly/removals.rs");
include!("graph_assembly/placements.rs");
include!("graph_assembly/reconstruction.rs");
include!("graph_assembly/imports.rs");
