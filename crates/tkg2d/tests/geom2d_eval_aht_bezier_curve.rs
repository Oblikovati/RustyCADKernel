//! Direct translation of Geom2dEval_AHTBezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{
    Geom2dBoundedCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry,
};
use rustycad_tkg2d::geom2d_eval::Geom2dEvalAHTBezierCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};

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

// Helper to create a curve with basis {1, t, sinh(alpha*t), cosh(alpha*t), sin(beta*t),
// cos(beta*t)} algDegree=1, alpha=1.0, beta=1.0 => basisDim = 2 + 2 + 2 = 6 poles needed
fn createFullBasisCurve() -> Geom2dEvalAHTBezierCurve {
    let aPoles = poles(&[
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (0.0, 1.0),
        (1.0, 0.0),
    ]);
    Geom2dEvalAHTBezierCurve::new(&aPoles, 1, 1.0, 1.0).unwrap()
}

// Helper to create a pure polynomial curve: basis {1, t, t^2} => 3 poles
fn createPolynomialCurve() -> Geom2dEvalAHTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    Geom2dEvalAHTBezierCurve::new(&aPoles, 2, 0.0, 0.0).unwrap()
}

// Helper to create a rational polynomial curve with weights
fn createRationalCurve() -> Geom2dEvalAHTBezierCurve {
    let aPoles = poles(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
    let aWeights = weights(&[1.0, 2.0, 1.0]);
    Geom2dEvalAHTBezierCurve::with_weights(&aPoles, &aWeights, 2, 0.0, 0.0).unwrap()
}

include!("geom2d_eval_aht_bezier_curve/suites_0.rs");
include!("geom2d_eval_aht_bezier_curve/suites_1.rs");
