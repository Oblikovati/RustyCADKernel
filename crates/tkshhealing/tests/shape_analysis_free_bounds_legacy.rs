//! All original ShapeAnalysis_FreeBounds_Test.cxx declarations at OCCT3d097a0.
#![allow(non_snake_case)]
mod healing_support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{Face, Shape, Vertex, Wire},
};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisFreeBounds;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
include!("free_bounds_legacy/support.rs");
include!("free_bounds_legacy/wires.rs");
include!("free_bounds_legacy/edges.rs");
include!("free_bounds_legacy/faces.rs");
