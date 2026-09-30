//! Direct translation of Geom2dEval_TBezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{
    Geom2dBoundedCurve, Geom2dCircle, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3,
    Geom2dEllipse, Geom2dGeometry,
};
use rustycad_tkg2d::geom2d_eval::Geom2dEvalTBezierCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis22d, Point2, Transform2, Vector2};
use std::f64::consts::PI;

const THE_FD_TOL_D1: f64 = 1e-5;
const THE_FD_TOL_D2: f64 = 1e-4;
const THE_FD_TOL_D3: f64 = 1e-3;

fn poles(points: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, &(x, y)) in points.iter().enumerate() {
        aPoles.set_value(i as i32 + 1, Point2::new(x, y)).unwrap();
    }
    aPoles
}

fn weights(values: &[f64]) -> Array1<'static, f64> {
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, &w) in values.iter().enumerate() {
        aWeights.set_value(i as i32 + 1, w).unwrap();
    }
    aWeights
}

// Helper: T-Bezier unit semicircle C(t) = (cos(t), sin(t)), t in [0, Pi].
fn createSemicircle2d() -> Geom2dEvalTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0), (0.0, 1.0), (1.0, 0.0)]);
    Geom2dEvalTBezierCurve::new(&aPoles, 1.0).unwrap()
}

// Helper: T-Bezier semi-ellipse (a=3, b=2): C(t) = (3*cos(t), 2*sin(t)).
fn createSemiEllipse2d() -> Geom2dEvalTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0), (0.0, 2.0), (3.0, 0.0)]);
    Geom2dEvalTBezierCurve::new(&aPoles, 1.0).unwrap()
}

// Helper to create a simple 3-pole (order 1) non-rational T-Bezier curve
fn createSimpleCurve() -> Geom2dEvalTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    Geom2dEvalTBezierCurve::new(&aPoles, 1.0).unwrap()
}

// Helper to create a 5-pole (order 2) non-rational T-Bezier curve
fn createQuadraticCurve() -> Geom2dEvalTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0), (3.0, -1.0), (4.0, 0.5)]);
    Geom2dEvalTBezierCurve::new(&aPoles, 0.5).unwrap()
}

include!("geom2d_eval_tbezier_curve/suites_0.rs");
include!("geom2d_eval_tbezier_curve/suites_1.rs");
