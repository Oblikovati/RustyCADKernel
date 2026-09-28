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

//! Direct translation of Geom2dGridEval_Hyperbola_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dHyperbola};
use rustycad_tkg2d::geom2d_grid_eval::{
    Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3, Geom2dGridEvalHyperbola,
};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2};

const THE_TOLERANCE: f64 = 1e-10;

// `new Geom2d_Hyperbola(gp_Ax22d(gp_Pnt2d(x, y), gp_Dir2d(1, 0)), 3.0, 2.0)`.
fn Hyperbola(x: f64, y: f64) -> Handle<Geom2dHyperbola> {
    let aAxis = Axis22d::new(Point2::new(x, y), Direction2::new(1.0, 0.0).unwrap(), true).unwrap();
    Handle::new(Geom2dHyperbola::new(aAxis, 3.0, 2.0).unwrap())
}

#[test]
fn Geom2dGridEval_HyperbolaTest__BasicD0() {
    let aHypr = Hyperbola(0.0, 0.0);

    let anEval = Geom2dGridEvalHyperbola::new(aHypr.clone());
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 11);

    for i in 1..=11 {
        let aExpected: Point2 = aHypr.value(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].distance(aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_HyperbolaTest__AtOrigin() {
    let aHypr = Hyperbola(0.0, 0.0);

    let anEval = Geom2dGridEvalHyperbola::new(aHypr);

    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 1).unwrap();
    aParams[1] = 0.0;

    let aGrid: Array1<Point2> = anEval.evaluate_grid(&aParams).unwrap();

    // At u=0: P = Center + MajR*cosh(0)*X + MinR*sinh(0)*Y = Center + MajR*X = (3, 0)
    EXPECT_NEAR(aGrid[1].x(), 3.0, THE_TOLERANCE);
    EXPECT_NEAR(aGrid[1].y(), 0.0, THE_TOLERANCE);
}

#[test]
fn Geom2dGridEval_HyperbolaTest__D1() {
    let aHypr = Hyperbola(1.0, 2.0);

    let anEval = Geom2dGridEvalHyperbola::new(aHypr.clone());

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1) = aHypr.d1(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_HyperbolaTest__D2() {
    let aHypr = Hyperbola(0.0, 0.0);

    let anEval = Geom2dGridEvalHyperbola::new(aHypr.clone());

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1, aD2) = aHypr.d2(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn Geom2dGridEval_HyperbolaTest__D3() {
    let aHypr = Hyperbola(0.0, 0.0);

    let anEval = Geom2dGridEvalHyperbola::new(aHypr.clone());

    let aParams = CreateUniformParams(-2.0, 2.0, 11);

    let aGrid: Array1<Geom2dGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    for i in 1..=11 {
        let (aPnt, aD1, aD2, aD3) = aHypr.d3(aParams[i]).unwrap();
        EXPECT_NEAR(aGrid[i].point.distance(aPnt), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d1 - aD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d2 - aD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aGrid[i].d3 - aD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}
