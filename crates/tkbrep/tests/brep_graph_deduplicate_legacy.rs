//! Legacy BRepGraph_Deduplicate_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::layer_history::LayerHistory;
use rustycad_tkbrep::brep_graph::shapes_view::Options as ShapeOptions;
use rustycad_tkbrep::brep_graph::{
    compact,
    deduplicate::{self, DeduplicateResult, Options},
    validate, *,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPICopy, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer as Analyzer;
include!("brep_graph_deduplicate_legacy/fixtures.rs");
include!("brep_graph_deduplicate_legacy/geometry.rs");
include!("brep_graph_deduplicate_legacy/basic_1.rs");
include!("brep_graph_deduplicate_legacy/basic_2.rs");
include!("brep_graph_deduplicate_legacy/basic_3.rs");
include!("brep_graph_deduplicate_legacy/history.rs");
include!("brep_graph_deduplicate_legacy/topology.rs");
include!("brep_graph_deduplicate_legacy/pipeline.rs");
include!("brep_graph_deduplicate_legacy/checks.rs");
