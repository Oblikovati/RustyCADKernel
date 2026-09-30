//! One-to-one port of BRepGraph_ShapesViewImport_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871, using production primitive/copy builders.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::child_explorer::ChildExplorer;
use rustycad_tkbrep::brep_graph::defs_iterator::DefsVertexOfEdge;
use rustycad_tkbrep::brep_graph::iterator::{EdgeIterator, FaceIterator, VertexIterator};
use rustycad_tkbrep::brep_graph::layer_history::LayerHistory;
use rustycad_tkbrep::brep_graph::related_iterator::{RelatedIterator, RelationKind};
use rustycad_tkbrep::brep_graph::shapes_view::{Options, ShapeNodeMap};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
use support::ref_test_tools::*;
const FORWARD: ParityOrientation = ParityOrientation::FORWARD;
include!("brep_graph_shapes_view_import_legacy/fixtures.rs");
include!("brep_graph_shapes_view_import_legacy/construction.rs");
include!("brep_graph_shapes_view_import_legacy/mutation.rs");
include!("brep_graph_shapes_view_import_legacy/adjacency.rs");
