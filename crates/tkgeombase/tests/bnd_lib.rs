//! Direct translation of BndLib_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The suites are split across `bnd_lib/suites_N.rs` in the upstream order.
#![allow(non_snake_case)]
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{array1, array2, curve, curve2d, dir, dir2d, pnt, pnt2d, surface};
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::{
    Geom2dBezierCurve, Geom2dCircle, Geom2dEllipse, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomBezierSurface, GeomCircle, GeomConicalSurface,
    GeomCylindricalSurface, GeomEllipse, GeomLine, GeomPlane, GeomSphericalSurface,
    GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::bnd_lib::{self, Add2dCurve, Add3dCurve, AddSurface};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Axis3, Circle2, Circle3, Cone3, Cylinder3, Direction3, Ellipse2, Ellipse3,
    Hyperbola2, Hyperbola3, Line2, Line3, Parabola2, Parabola3, Point3, Sphere3, Torus3,
};
use std::f64::consts::PI;

// `gp_Ax2(P, V)`.
fn ax2(theP: Point3, theV: Direction3) -> Axis2 {
    Axis2::new(theP, theV).unwrap()
}

// `gp_Ax3(P, V)`.
fn ax3(theP: Point3, theV: Direction3) -> Axis3 {
    Axis3::from_normal(theP, theV).unwrap()
}

// `gp_Ax2d(P, V)`.
fn ax2d(theX: f64, theY: f64, theDx: f64, theDy: f64) -> Axis2d {
    Axis2d::new(pnt2d(theX, theY), dir2d(theDx, theDy))
}

// The legacy `Get(Xmin, Ymin, Zmin, Xmax, Ymax, Zmax)`.
fn get(theBox: &BoundingBox3) -> [f64; 6] {
    theBox.get().unwrap()
}

// The legacy `Get(Xmin, Ymin, Xmax, Ymax)`.
fn get2d(theBox: &BoundingBox2) -> [f64; 4] {
    theBox.get().unwrap()
}

include!("bnd_lib/suites_0.rs");
include!("bnd_lib/suites_1.rs");
include!("bnd_lib/suites_2.rs");
include!("bnd_lib/suites_3.rs");
