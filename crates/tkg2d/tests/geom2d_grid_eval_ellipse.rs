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

//! Direct translation of Geom2dGridEval_Ellipse_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dEllipse};
use rustycad_tkg2d::geom2d_grid_eval::{
    Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3, Geom2dGridEvalEllipse,
};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-10;

// `new Geom2d_Ellipse(gp_Ax22d(gp_Pnt2d(x, y), gp_Dir2d(1, 0)), a, b)`.
fn Ellipse(x: f64, y: f64, a: f64, b: f64) -> Handle<Geom2dEllipse> {
    let aAxis = Axis22d::new(Point2::new(x, y), Direction2::new(1.0, 0.0).unwrap(), true).unwrap();
    Handle::new(Geom2dEllipse::new(aAxis, a, b).unwrap())
}

#[test]
fn Geom2dGridEval_EllipseTest__BasicD0() {
    let anEllipse = Ellipse(0.0, 0.0, 3.0, 2.0);

    let anEval = Geom2dGridEvalEllipse::new(anEllipse.clone());
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 13);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 13);

    for i in 1..=13 {
        let aExpected: Point2 = anEllipse.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_EllipseTest__CardinalPoints() {
    let anEllipse = Ellipse(0.0, 0.0, 3.0, 2.0);

    let anEval = Geom2dGridEvalEllipse::new(anEllipse);

    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    aParams[1] = 0.0;
    aParams[2] = PI / 2.0;
    aParams[3] = PI;
    aParams[4] = 3.0 * PI / 2.0;

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    // u=0: (3, 0)
    EXPECT_NEAR(aGrid[1].x(), 3.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[1].y(), 0.0, THE_TOLERANCE);

    // u=PI/2: (0, 2)
    EXPECT_NEAR(aGrid[2].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[2].y(), 2.0, THE_TOLERANCE);

    // u=PI: (-3, 0)
    EXPECT_NEAR(aGrid[3].x(), -3.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[3].y(), 0.0, THE_TOLERANCE);

    // u=3PI/2: (0, -2)
    EXPECT_NEAR(aGrid[4].x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[4].y(), -2.0, THE_TOLERANCE);
}

#[test]
fn Geom2dGridEval_EllipseTest__D1() {
    let anEllipse = Ellipse(1.0, 2.0, 5.0, 3.0);

    let anEval = Geom2dGridEvalEllipse::new(anEllipse.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 13);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=13 {
        let (aPnt, aD1) = anEllipse.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_EllipseTest__D2() {
    let anEllipse = Ellipse(0.0, 0.0, 3.0, 2.0);

    let anEval = Geom2dGridEvalEllipse::new(anEllipse.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Geom2dGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2) = anEllipse.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_EllipseTest__D3() {
    let anEllipse = Ellipse(0.0, 0.0, 3.0, 2.0);

    let anEval = Geom2dGridEvalEllipse::new(anEllipse.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=9 {
        let (aPnt, aD1, aD2, aD3) = anEllipse.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}
