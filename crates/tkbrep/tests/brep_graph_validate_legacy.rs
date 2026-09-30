//! Original BRepGraph_Validate_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{
        BRepGraph, CoEdgeId, EdgeCurve3DRepId, EdgeId, FaceId, NodeId, NodeKind, OccurrenceId,
        ProductId, VertexId, VertexRefId, WireId, deduplicate,
        iterator::EdgeIterator,
        validate::{self, Options, Severity, ValidationResult},
    },
    brep_graph_inc::ParityOrientation,
    topexp::Explorer,
    topods::{Compound, Shape},
};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg3d::{
    geom::{
        GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSphericalSurface,
        GeomSurface,
    },
    top_abs::ShapeEnum,
};
use rustycad_tkmath::{
    gp::{Axis1, Axis2, Axis3, Direction3, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
include!("graph_validate/fixtures.rs");
include!("graph_validate/clean.rs");
include!("graph_validate/corruption.rs");
include!("graph_validate/assembly.rs");
include!("graph_validate/connectivity.rs");
include!("graph_validate/synthetic_helpers.rs");
include!("graph_validate/synthetic_box.rs");
include!("graph_validate/synthetic_curved.rs");
include!("graph_validate/synthetic_assembly.rs");
