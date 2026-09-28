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

//! Direct translation of GeomGridEval_BezierSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomSurface};
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalBezierSurface, GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3,
};
use rustycad_tkmath::gp::{Point3, Vector3};

const THE_TOLERANCE: f64 = 1e-10;

// The 2x2 poles (0,0,0), (1,0,0), (0,1,0), (1,1,theCornerZ) set as the suite sets them.
fn Poles2x2(theCornerZ: f64) -> Array2<'static, Point3> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aPoles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    aPoles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    aPoles[(1, 2)] = Point3::new(0.0, 1.0, 0.0);
    aPoles[(2, 2)] = Point3::new(1.0, 1.0, theCornerZ);
    aPoles
}

// The 2x2 weights 1, 1, 1, 2 of the rational tests (higher weight on the corner).
fn Weights2x2() -> Array2<'static, f64> {
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aWeights[(1, 1)] = 1.0;
    aWeights[(2, 1)] = 1.0;
    aWeights[(1, 2)] = 1.0;
    aWeights[(2, 2)] = 2.0;
    aWeights
}

// The 3x3 poles `gp_Pnt(i, j, std::sin(i + j))`.
fn SinPoles3x3() -> Array2<'static, Point3> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aPoles[(i, j)] = Point3::new(f64::from(i), f64::from(j), f64::from(i + j).sin());
        }
    }
    aPoles
}

// The n x n poles `gp_Pnt(i - 1, j - 1, std::sin((i - 1) * 0.5 + (j - 1) * 0.5))`.
fn ShiftedSinPoles(theNb: i32) -> Array2<'static, Point3> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, theNb, 1, theNb).unwrap();
    for i in 1..=theNb {
        for j in 1..=theNb {
            let (x, y) = (f64::from(i - 1), f64::from(j - 1));
            aPoles[(i, j)] = Point3::new(x, y, (x * 0.5 + y * 0.5).sin());
        }
    }
    aPoles
}

fn Bezier(thePoles: &Array2<'_, Point3>) -> Handle<GeomBezierSurface> {
    Handle::new(GeomBezierSurface::new(thePoles).unwrap())
}

fn RationalBezier(
    thePoles: &Array2<'_, Point3>,
    theWeights: &Array2<'_, f64>,
) -> Handle<GeomBezierSurface> {
    Handle::new(GeomBezierSurface::with_weights(thePoles, theWeights).unwrap())
}

// A one-element parameter array `aParams(1, 1)` holding `theValue`.
fn SingleParam(theValue: f64) -> Array1<'static, f64> {
    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 1).unwrap();
    aParams[1] = theValue;
    aParams
}

// The DN loop of the DN tests: the grid DN of order `(theNU, theNV)` on 5 x 5 uniform
// parameters in [0, 1] against `Geom_BezierSurface::DN`.
fn CheckGridDN(theBezier: &Handle<GeomBezierSurface>, theNU: i32, theNV: i32) {
    let anEval = GeomGridEvalBezierSurface::new(theBezier.clone());
    let aParams = CreateUniformParams(0.0, 1.0, 5);
    let aGrid: Array2<Vector3> = anEval
        .evaluate_grid_dn(&aParams, &aParams, theNU, theNV)
        .unwrap();
    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Vector3 = theBezier.dn(aParams[i], aParams[j], theNU, theNV).unwrap();
            EXPECT_NEAR((aGrid[(i, j)] - aExpected).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__BasicEvaluation() {
    // Simple Bezier surface 2x2 (planar)
    let aBezier = Bezier(&Poles2x2(0.0));

    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());
    assert!(!anEval.geometry().is_null());

    let aParams = CreateUniformParams(0.0, 1.0, 3);
    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aParams, &aParams).unwrap();

    // RowLength() is the number of columns, ColLength() the number of rows.
    assert_eq!(aGrid.nb_columns(), 3);
    assert_eq!(aGrid.nb_rows(), 3);

    // Verify points
    for i in 1..=3 {
        for j in 1..=3 {
            let aExpected: Point3 = aBezier.value(aParams[i], aParams[j]).unwrap();
            EXPECT_NEAR(aGrid[(i, j)].distance(&aExpected), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__RationalEvaluation() {
    // Rational Bezier surface
    // Elevated corner, higher weight
    let aBezier = RationalBezier(&Poles2x2(1.0), &Weights2x2());
    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);
    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aParams, &aParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Point3 = aBezier.value(aParams[i], aParams[j]).unwrap();
            EXPECT_NEAR(aGrid[(i, j)].distance(&aExpected), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeD1() {
    let aBezier = Bezier(&Poles2x2(1.0));
    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);
    let aGrid: Array2<GeomGridEvalSurfD1> = anEval.evaluate_grid_d1(&aParams, &aParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = aBezier.d1(aParams[i], aParams[j]).unwrap();
            let g = &aGrid[(i, j)];
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeD2() {
    let aBezier = Bezier(&SinPoles3x3());
    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);
    let aGrid: Array2<GeomGridEvalSurfD2> = anEval.evaluate_grid_d2(&aParams, &aParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = aBezier.d2(aParams[i], aParams[j]).unwrap();
            let g = &aGrid[(i, j)];
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeD3() {
    let aBezier = Bezier(&ShiftedSinPoles(4));
    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 5);
    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aParams, &aParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = aBezier.d3(aParams[i], aParams[j]).unwrap();
            let g = &aGrid[(i, j)];
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3u - r.d3u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3v - r.d3v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uuv - r.d3uuv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uvv - r.d3uvv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeDN_U1V0() {
    CheckGridDN(&Bezier(&SinPoles3x3()), 1, 0);
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeDN_U0V1() {
    CheckGridDN(&Bezier(&SinPoles3x3()), 0, 1);
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeDN_U2V0() {
    CheckGridDN(&Bezier(&SinPoles3x3()), 2, 0);
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeDN_U1V1() {
    CheckGridDN(&Bezier(&SinPoles3x3()), 1, 1);
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeDN_BeyondDegree() {
    // Biquadratic Bezier (degree 2 in both directions)
    let aBezier = Bezier(&SinPoles3x3());
    let anEval = GeomGridEvalBezierSurface::new(aBezier);

    let aParams = CreateUniformParams(0.0, 1.0, 5);

    // 3rd derivative in U direction (beyond degree 2) should be zero
    let aGrid: Array2<Vector3> = anEval.evaluate_grid_dn(&aParams, &aParams, 3, 0).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            EXPECT_NEAR(aGrid[(i, j)].magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__DerivativeDN_RationalSurface() {
    // Rational Bezier surface
    let aBezier = RationalBezier(&Poles2x2(1.0), &Weights2x2());

    // Test DN(1,0), DN(0,1), and DN(1,1)
    for aNU in 0..=1 {
        for aNV in 0..=1 {
            if aNU + aNV == 0 {
                continue;
            }
            CheckGridDN(&aBezier, aNU, aNV);
        }
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__IsolineU_CompareToGeomD0() {
    // Create a non-planar Bezier surface
    let aBezier = Bezier(&ShiftedSinPoles(3));
    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());

    // U-isoline: 1 U param, multiple V params (triggers isoline path)
    let aUParams = SingleParam(0.5);
    let aVParams = CreateUniformParams(0.0, 1.0, 10);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Note: RowLength() = V count (columns), ColLength() = U count (rows)
    assert_eq!(aGrid.nb_columns(), 10);
    assert_eq!(aGrid.nb_rows(), 1);

    // Compare against Geom_Surface::D0
    for j in 1..=10 {
        let aExpected: Point3 = aBezier.d0(aUParams[1], aVParams[j]).unwrap();
        EXPECT_NEAR(aGrid[(1, j)].distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__IsolineV_CompareToGeomD0() {
    // Create a non-planar Bezier surface
    let aBezier = Bezier(&ShiftedSinPoles(3));
    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());

    // V-isoline: multiple U params, 1 V param (triggers isoline path)
    let aUParams = CreateUniformParams(0.0, 1.0, 10);
    let aVParams = SingleParam(0.7);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Note: RowLength() = V count (columns), ColLength() = U count (rows)
    assert_eq!(aGrid.nb_columns(), 1);
    assert_eq!(aGrid.nb_rows(), 10);

    // Compare against Geom_Surface::D0
    for i in 1..=10 {
        let aExpected: Point3 = aBezier.d0(aUParams[i], aVParams[1]).unwrap();
        EXPECT_NEAR(aGrid[(i, 1)].distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BezierSurfaceTest__IsolineRational_CompareToGeomD0() {
    // Rational Bezier surface
    let aPoles = ShiftedSinPoles(3);
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            aWeights[(i, j)] = 1.0 + 0.5 * f64::from((i - 1) + (j - 1));
        }
    }
    let aBezier = RationalBezier(&aPoles, &aWeights);
    let anEval = GeomGridEvalBezierSurface::new(aBezier.clone());

    // U-isoline on rational surface
    let aUParams = SingleParam(0.3);
    let aVParams = CreateUniformParams(0.0, 1.0, 15);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for j in 1..=15 {
        let aExpected: Point3 = aBezier.d0(aUParams[1], aVParams[j]).unwrap();
        EXPECT_NEAR(aGrid[(1, j)].distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}
