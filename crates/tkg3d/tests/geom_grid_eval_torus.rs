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

//! Direct translation of GeomGridEval_Torus_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomSurface, GeomToroidalSurface};
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3, GeomGridEvalTorus,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-10;

// `new Geom_ToroidalSurface(gp_Ax3(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 10.0, 2.0)`.
fn MakeTorus() -> Handle<GeomToroidalSurface> {
    let anAxis = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Handle::new(GeomToroidalSurface::new(anAxis, 10.0, 2.0).unwrap())
}

//=================================================================================================
// Grid evaluation tests
//=================================================================================================

#[test]
fn GeomGridEval_TorusTest__GridBasicEvaluation() {
    // Torus: Major=10, Minor=2, Center(0,0,0), Z-axis
    let aTorus = MakeTorus();

    let anEval = GeomGridEvalTorus::new(aTorus.clone());
    assert!(!anEval.geometry().is_null());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9); // Major
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9); // Minor

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();
    // RowLength() is the number of columns, ColLength() the number of rows.
    assert_eq!(aGrid.nb_columns(), 9);
    assert_eq!(aGrid.nb_rows(), 9);

    // Verify points
    for iU in 1..=9 {
        for iV in 1..=9 {
            let aExpected: Point3 = aTorus.value(aUParams[iU], aVParams[iV]).unwrap();
            EXPECT_NEAR(
                aGrid.value(iU, iV).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_TorusTest__GridDerivativeD1() {
    let aTorus = MakeTorus();
    let anEval = GeomGridEvalTorus::new(aTorus.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array2<GeomGridEvalSurfD1> = anEval.evaluate_grid_d1(&aUParams, &aVParams).unwrap();

    for iU in 1..=9 {
        for iV in 1..=9 {
            let r = aTorus.d1(aUParams[iU], aVParams[iV]).unwrap();
            let g = aGrid.value(iU, iV).unwrap();
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_TorusTest__GridDerivativeD2() {
    let aTorus = MakeTorus();
    let anEval = GeomGridEvalTorus::new(aTorus.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array2<GeomGridEvalSurfD2> = anEval.evaluate_grid_d2(&aUParams, &aVParams).unwrap();

    for iU in 1..=9 {
        for iV in 1..=9 {
            let r = aTorus.d2(aUParams[iU], aVParams[iV]).unwrap();
            let g = aGrid.value(iU, iV).unwrap();
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
fn GeomGridEval_TorusTest__GridDerivativeD3() {
    let aTorus = MakeTorus();
    let anEval = GeomGridEvalTorus::new(aTorus.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    for iU in 1..=9 {
        for iV in 1..=9 {
            let r = aTorus.d3(aUParams[iU], aVParams[iV]).unwrap();
            let g = aGrid.value(iU, iV).unwrap();
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3u - r.d3u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3v - r.d3v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uuv - r.d3uuv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uvv - r.d3uvv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_TorusTest__GridDerivativeDN() {
    let aTorus = MakeTorus();
    let anEval = GeomGridEvalTorus::new(aTorus.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 5);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 4);

    // Test D4U (4th derivative in U)
    let aD4U: Array2<Vector3> = anEval.evaluate_grid_dn(&aUParams, &aVParams, 4, 0).unwrap();
    for iU in 1..=5 {
        for iV in 1..=4 {
            let aExpected: Vector3 = aTorus.dn(aUParams[iU], aVParams[iV], 4, 0).unwrap();
            EXPECT_NEAR(
                (*aD4U.value(iU, iV).unwrap() - aExpected).magnitude(),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}
