//! Direct translation of Geom2d_BSplineCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};

// The fixture SetUp: a cubic Bezier-like B-spline through (0,0), (1,1), (2,1), (3,0).
fn SetUp() -> Handle<Geom2dBSplineCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(2.0, 1.0);
    aPoles[4] = Point2::new(3.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

// Knots {0, 1} with both multiplicities `mult`, as several tests build them.
fn end_knots(mult: i32) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = mult;
    aMults[2] = mult;
    (aKnots, aMults)
}

fn poles3(p1: Point2, p2: Point2, p3: Point2) -> Array1<'static, Point2> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 3).unwrap();
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

include!("geom2d_bspline_curve/suites_0.rs");
include!("geom2d_bspline_curve/suites_1.rs");
