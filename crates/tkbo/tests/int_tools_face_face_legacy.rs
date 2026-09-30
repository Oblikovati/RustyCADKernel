//! All original IntTools_FaceFace_Test.cxx cases, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, clippy::excessive_precision)]
use rustycad_tkbo::int_tools::FaceFace;
use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomPlane, GeomSurface,
    GeomSurfaceOfLinearExtrusion,
};
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use std::f64::consts::{PI, TAU};
include!("face_face_legacy/fixtures.rs");
include!("face_face_legacy/circular.rs");
include!("face_face_legacy/regressions.rs");
