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

//! Direct translation of GeomGridEval_SurfaceOfRevolution_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomCircle, GeomLine, GeomSurface, GeomSurfaceOfRevolution};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3, GeomGridEvalSurface,
    GeomGridEvalSurfaceOfRevolution,
};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-9;

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

// `new Geom_Line(gp_Pnt(5.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0))`.
fn Meridian() -> Handle<GeomLine> {
    Handle::new(GeomLine::from_point(
        Point3::new(5.0, 0.0, 0.0),
        dir(0.0, 0.0, 1.0),
    ))
}

// `gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1))`.
fn ZAxis() -> Axis1 {
    Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0))
}

// `new Geom_SurfaceOfRevolution(theCurve, theAxis)`.
fn Revolution<C: RuntimeTyped + ?Sized>(
    theCurve: &Handle<C>,
    theAxis: Axis1,
) -> Handle<GeomSurfaceOfRevolution> {
    Handle::new(GeomSurfaceOfRevolution::new(&theCurve.cast(), theAxis).unwrap())
}

#[test]
fn GeomGridEval_SurfaceOfRevolutionTest__BasicEvaluation() {
    // Create a line parallel to Y axis at X=5 (meridian)
    let aLine = Meridian();

    // Revolve around Z axis -> Creates a cylinder of radius 5
    let aRevAxis = ZAxis();
    let aRevSurf = Revolution(&aLine, aRevAxis);

    let anEval = GeomGridEvalSurfaceOfRevolution::new(aRevSurf.clone());

    let aUParams = CreateUniformParams(0.0, PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=9 {
        for j in 1..=5 {
            let aExpected: Point3 = aRevSurf.value(aUParams[i], aVParams[j]).unwrap();
            let aPnt: Point3 = *aGrid.value(i, j).unwrap();
            EXPECT_NEAR(aPnt.distance(&aExpected), 0.0, THE_TOLERANCE);

            // Check cylindrical radius
            let aRadius: f64 = (aPnt.x() * aPnt.x() + aPnt.y() * aPnt.y()).sqrt();
            EXPECT_NEAR(aRadius, 5.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfRevolutionTest__CircleMeridian() {
    // Create a circle in XZ plane with center at (10, 0, 0) and radius 3
    // This creates a torus when revolved
    let aCircleAx = Axis2::with_x_direction(
        Point3::new(10.0, 0.0, 0.0),
        dir(0.0, 1.0, 0.0),
        dir(1.0, 0.0, 0.0),
    )
    .unwrap();
    let aCircle = Handle::new(GeomCircle::new(aCircleAx, 3.0).unwrap());

    // Revolve around Z axis
    let aRevAxis = ZAxis();
    let aRevSurf = Revolution(&aCircle, aRevAxis);

    let anEval = GeomGridEvalSurfaceOfRevolution::new(aRevSurf.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=9 {
        for j in 1..=9 {
            let aExpected: Point3 = aRevSurf.value(aUParams[i], aVParams[j]).unwrap();
            EXPECT_NEAR(
                aGrid.value(i, j).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfRevolutionTest__DerivativeD1() {
    let aLine = Meridian();
    let aRevAxis = ZAxis();
    let aRevSurf = Revolution(&aLine, aRevAxis);

    let anEval = GeomGridEvalSurfaceOfRevolution::new(aRevSurf.clone());

    let aUParams = CreateUniformParams(0.0, PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGridD1: Array2<GeomGridEvalSurfD1> =
        anEval.evaluate_grid_d1(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = aRevSurf.d1(aUParams[i], aVParams[j]).unwrap();

            let aData = aGridD1.value(i, j).unwrap();
            EXPECT_NEAR(aData.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfRevolutionTest__DerivativeD2() {
    let aLine = Meridian();
    let aRevAxis = ZAxis();
    let aRevSurf = Revolution(&aLine, aRevAxis);

    let anEval = GeomGridEvalSurfaceOfRevolution::new(aRevSurf.clone());

    let aUParams = CreateUniformParams(0.0, PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGridD2: Array2<GeomGridEvalSurfD2> =
        anEval.evaluate_grid_d2(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = aRevSurf.d2(aUParams[i], aVParams[j]).unwrap();

            let aData = aGridD2.value(i, j).unwrap();
            EXPECT_NEAR(aData.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfRevolutionTest__DerivativeD3() {
    let aLine = Meridian();
    let aRevAxis = ZAxis();
    let aRevSurf = Revolution(&aLine, aRevAxis);

    let anEval = GeomGridEvalSurfaceOfRevolution::new(aRevSurf.clone());

    let aUParams = CreateUniformParams(0.0, PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGridD3: Array2<GeomGridEvalSurfD3> =
        anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = aRevSurf.d3(aUParams[i], aVParams[j]).unwrap();

            let aData = aGridD3.value(i, j).unwrap();
            EXPECT_NEAR(aData.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3u - r.d3u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3v - r.d3v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3uuv - r.d3uuv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3uvv - r.d3uvv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfRevolutionTest__UnifiedDispatch() {
    let aLine = Meridian();
    let aRevAxis = ZAxis();
    let aRevSurf = Revolution(&aLine, aRevAxis);

    // Test dispatch via unified evaluator
    let anEval = GeomGridEvalSurface::from_surface(&aRevSurf.cast());
    assert_eq!(anEval.get_type(), SurfaceType::SurfaceOfRevolution);

    let aUParams = CreateUniformParams(0.0, PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Point3 = aRevSurf.value(aUParams[i], aVParams[j]).unwrap();
            EXPECT_NEAR(
                aGrid.value(i, j).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfRevolutionTest__AdaptorDispatch() {
    let aLine = Meridian();
    let aRevAxis = ZAxis();
    let aRevSurf = Revolution(&aLine, aRevAxis);

    // Test dispatch via adaptor
    let anAdaptor = GeomAdaptorSurface::with_surface(&aRevSurf.cast()).unwrap();
    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::SurfaceOfRevolution);

    let aUParams = CreateUniformParams(0.0, PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Point3 = aRevSurf.value(aUParams[i], aVParams[j]).unwrap();
            EXPECT_NEAR(
                aGrid.value(i, j).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}
