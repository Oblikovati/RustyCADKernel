// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of GeomGridEval_Curve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalBSplineCurve, GeomGridEvalCircle, GeomGridEvalCurve, GeomGridEvalCurveD1,
    GeomGridEvalCurveD2, GeomGridEvalCurveD3, GeomGridEvalLine, GeomGridEvalOtherCurve,
};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-10;

// Helper function to create a simple B-spline curve
fn CreateSimpleBSpline() -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(3.0, 2.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    aMults[1] = 4;
    aMults[2] = 4;

    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

// `gp_Dir(x, y, z)`.
fn Dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

// `gp_Ax2(gp_Pnt(x, y, z), gp_Dir(dx, dy, dz))`.
fn Ax2(x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64) -> Axis2 {
    Axis2::new(Point3::new(x, y, z), Dir(dx, dy, dz)).unwrap()
}

// `new Geom_Line(gp_Pnt(x, y, z), gp_Dir(dx, dy, dz))`.
fn Line(x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64) -> Handle<GeomLine> {
    Handle::new(GeomLine::from_point(Point3::new(x, y, z), Dir(dx, dy, dz)))
}

// `new Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.0)`.
fn CircleOfRadius2() -> Handle<GeomCircle> {
    Handle::new(GeomCircle::new(Ax2(0.0, 0.0, 0.0, 0.0, 0.0, 1.0), 2.0).unwrap())
}

// `new Geom_OffsetCurve(theBasis, theOffset, gp::DZ())`.
fn OffsetDZ<T: GeomCurve + RuntimeTyped>(
    theBasis: &Handle<T>,
    theOffset: f64,
) -> Handle<GeomOffsetCurve> {
    let aBasis: Handle<dyn GeomCurve> = theBasis.cast();
    Handle::new(GeomOffsetCurve::new(&aBasis, theOffset, Dir(0.0, 0.0, 1.0), false).unwrap())
}

// `GeomAdaptor_Curve anAdaptor(theCurve)`.
fn Adaptor<T: GeomCurve + RuntimeTyped>(theCurve: &Handle<T>) -> GeomAdaptorCurve {
    GeomAdaptorCurve::with_curve(&theCurve.cast::<dyn GeomCurve>()).unwrap()
}

// The rational B-spline (NURBS circle approximation) shared by RationalBSpline and
// DerivativeDN_RationalCurve.
fn CreateRationalBSpline() -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();

    aPoles[1] = Point3::new(1.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 1.0, 0.0);
    aPoles[3] = Point3::new(0.0, 1.0, 0.0);
    aPoles[4] = Point3::new(-1.0, 1.0, 0.0);

    aWeights[1] = 1.0;
    aWeights[2] = 1.0 / 2.0_f64.sqrt();
    aWeights[3] = 1.0;
    aWeights[4] = 1.0 / 2.0_f64.sqrt();

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    aMults[1] = 4;
    aMults[2] = 4;

    Handle::new(
        GeomBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 3, false).unwrap(),
    )
}

// The multi-span B-spline (degree 3, 6 control points, 1 internal knot) shared by
// MultiSpanBSpline and DerivativeDN_MultiSpan.
fn CreateMultiSpanBSpline() -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 6).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(2.0, 2.0, 0.0);
    aPoles[4] = Point3::new(3.0, 0.0, 0.0);
    aPoles[5] = Point3::new(4.0, -1.0, 0.0);
    aPoles[6] = Point3::new(5.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 3).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 0.5;
    aKnots[3] = 1.0;
    aMults[1] = 4;
    aMults[2] = 2;
    aMults[3] = 4;

    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

// `NCollection_Array1<double> aParams(1, 5)` holding 0, PI/2, PI, 3PI/2, 2PI.
fn QuarterTurnParams() -> Array1<'static, f64> {
    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    aParams[1] = 0.0;
    aParams[2] = PI / 2.0;
    aParams[3] = PI;
    aParams[4] = 3.0 * PI / 2.0;
    aParams[5] = 2.0 * PI;
    aParams
}

include!("geom_grid_eval_curve/suites_0.rs");
include!("geom_grid_eval_curve/suites_1.rs");
