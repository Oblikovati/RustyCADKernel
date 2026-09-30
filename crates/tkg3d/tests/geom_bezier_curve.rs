//! Direct translation of Geom_BezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomBoundedCurve, GeomCurve, GeomGeometry};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

// The fixture SetUp: a cubic Bezier curve through (0,0,0), (1,1,0), (2,1,0), (3,0,0).
fn SetUp() -> Handle<GeomBezierCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = pnt(0.0, 0.0, 0.0);
    aPoles[2] = pnt(1.0, 1.0, 0.0);
    aPoles[3] = pnt(2.0, 1.0, 0.0);
    aPoles[4] = pnt(3.0, 0.0, 0.0);
    Handle::new(GeomBezierCurve::new(&aPoles).unwrap())
}

fn poles3(p1: Point3, p2: Point3, p3: Point3) -> Array1<'static, Point3> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = p1;
    aPoles[2] = p2;
    aPoles[3] = p3;
    aPoles
}

fn weights3(w1: f64, w2: f64, w3: f64) -> Array1<'static, f64> {
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aWeights[1] = w1;
    aWeights[2] = w2;
    aWeights[3] = w3;
    aWeights
}

// The `new Geom_BezierCurve(*curve)` of the copy tests.
fn copied(curve: &GeomBezierCurve) -> Handle<GeomBezierCurve> {
    Handle::new(GeomBezierCurve::copy_of(curve).unwrap())
}

include!("geom_bezier_curve/suites_0.rs");
include!("geom_bezier_curve/suites_1.rs");
