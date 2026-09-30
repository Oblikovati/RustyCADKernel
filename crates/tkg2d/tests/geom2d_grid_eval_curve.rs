//! Direct translation of Geom2dGridEval_Curve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg2d::geom2d_grid_eval::{
    Geom2dGridEvalBSplineCurve, Geom2dGridEvalCircle, Geom2dGridEvalCurve, Geom2dGridEvalCurveD1,
    Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3, Geom2dGridEvalLine,
};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2, Vector2};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-10;

// Helper function to create a simple 2D B-spline curve
fn CreateSimpleBSpline2d() -> Handle<Geom2dBSplineCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 2.0);
    aPoles[3] = Point2::new(3.0, 2.0);
    aPoles[4] = Point2::new(4.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    aMults[1] = 4;
    aMults[2] = 4;

    Handle::new(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

// `gp_Dir2d(x, y)`.
fn Dir(x: f64, y: f64) -> Direction2 {
    Direction2::new(x, y).unwrap()
}

// `gp_Ax22d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0))`.
fn OriginAxis() -> Axis22d {
    Axis22d::new(Point2::new(0.0, 0.0), Dir(1.0, 0.0), true).unwrap()
}

// `new Geom2d_Line(gp_Pnt2d(x, y), gp_Dir2d(dx, dy))`.
fn Line(x: f64, y: f64, dx: f64, dy: f64) -> Handle<Geom2dLine> {
    Handle::new(Geom2dLine::from_point(Point2::new(x, y), Dir(dx, dy)))
}

// `new Geom2d_Circle(gp_Ax22d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 2.0)`.
fn CircleOfRadius2() -> Handle<Geom2dCircle> {
    Handle::new(Geom2dCircle::new(OriginAxis(), 2.0).unwrap())
}

// `Geom2dAdaptor_Curve anAdaptor(theCurve)`.
fn Adaptor<T: Geom2dCurve + RuntimeTyped>(theCurve: &Handle<T>) -> Geom2dAdaptorCurve {
    Geom2dAdaptorCurve::with_curve(&theCurve.cast::<dyn Geom2dCurve>()).unwrap()
}

include!("geom2d_grid_eval_curve/suites_0.rs");
include!("geom2d_grid_eval_curve/suites_1.rs");
