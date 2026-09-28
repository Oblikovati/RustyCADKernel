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

//! Direct translation of Geom2dGridEval_BezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_grid_eval::{
    Geom2dGridEvalBezierCurve, Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3,
};
use rustycad_tkmath::gp::{Point2, Vector2};

const THE_TOLERANCE: f64 = 1e-10;

// The cubic Bezier curve that the first five tests build.
fn CubicBezier() -> Handle<Geom2dBezierCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 2.0);
    aPoles[3] = Point2::new(3.0, 2.0);
    aPoles[4] = Point2::new(4.0, 0.0);
    Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap())
}

#[test]
fn Geom2dGridEval_BezierCurveTest__BasicD0() {
    let aBezier = CubicBezier();

    let anEval = Geom2dGridEvalBezierCurve::new(aBezier.clone());
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);

    for i in 1..=11 {
        let aExpected: Point2 = aBezier.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BezierCurveTest__D1() {
    let aBezier = CubicBezier();

    let anEval = Geom2dGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1): (Point2, Vector2) = aBezier.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BezierCurveTest__D2() {
    let aBezier = CubicBezier();

    let anEval = Geom2dGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1, aD2) = aBezier.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BezierCurveTest__D3() {
    let aBezier = CubicBezier();

    let anEval = Geom2dGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1, aD2, aD3) = aBezier.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BezierCurveTest__DN_BeyondDegree() {
    let aBezier = CubicBezier();

    let anEval = Geom2dGridEvalBezierCurve::new(aBezier);

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    // Degree 3 Bezier, 4th derivative should be zero
    let aGrid: Array1<Vector2> = anEval.evaluate_grid_dn(&aParams, 4).unwrap();

    for i in 1..=11 {
        EXPECT_NEAR(aGrid[i].magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_BezierCurveTest__RationalBezier() {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 3).unwrap();
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();

    aPoles[1] = Point2::new(1.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(0.0, 1.0);

    aWeights[1] = 1.0;
    aWeights[2] = 1.0 / 2.0_f64.sqrt();
    aWeights[3] = 1.0;

    let aBezier = Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());

    let anEval = Geom2dGridEvalBezierCurve::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 21);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    for i in 1..=21 {
        let aExpected: Point2 = aBezier.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}
