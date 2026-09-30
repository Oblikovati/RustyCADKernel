//! Original declarations and inputs from ShapeConstruct_ProjectCurveOnSurface_Test.cxx
//! at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    precision,
    standard::Handle,
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::*;
use rustycad_tkgeomalgo::geom_api::PointsToBSpline;
use rustycad_tkgeombase::gc::MakeSegment;
use rustycad_tkmath::{
    geom_abs::Shape,
    gp::{Axis2, Axis3, Direction3, Point3},
};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisSurface, shape_construct::ShapeConstructProjectCurveOnSurface,
    shape_extend::ShapeExtendStatus,
};
use std::f64::consts::PI;
include!("project_curve_legacy/support.rs");
include!("project_curve_legacy/analytical.rs");
include!("project_curve_legacy/splines.rs");
include!("project_curve_legacy/state.rs");
include!("project_curve_legacy/edge_cases.rs");
include!("project_curve_legacy/bug27569.rs");
