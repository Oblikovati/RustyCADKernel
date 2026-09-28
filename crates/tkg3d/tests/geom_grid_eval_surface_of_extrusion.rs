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

//! Direct translation of GeomGridEval_SurfaceOfExtrusion_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomLine, GeomSurface, GeomSurfaceOfLinearExtrusion};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3, GeomGridEvalSurface,
    GeomGridEvalSurfaceOfExtrusion,
};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-9;

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

// `new Geom_Line(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(1.0, 0.0, 0.0))`.
fn LineAlongX() -> Handle<GeomLine> {
    Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(1.0, 0.0, 0.0),
    ))
}

// `new Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1), gp_Dir(1, 0, 0)), theRadius)`.
fn CircleXY(theRadius: f64) -> Handle<GeomCircle> {
    let aCircleAx = Axis2::with_x_direction(
        Point3::new(0.0, 0.0, 0.0),
        dir(0.0, 0.0, 1.0),
        dir(1.0, 0.0, 0.0),
    )
    .unwrap();
    Handle::new(GeomCircle::new(aCircleAx, theRadius).unwrap())
}

// `new Geom_SurfaceOfLinearExtrusion(theCurve, theDirection)`.
fn Extrusion<C: rustycad_tkernel::standard::RuntimeTyped + ?Sized>(
    theCurve: &Handle<C>,
    theDirection: Direction3,
) -> Handle<GeomSurfaceOfLinearExtrusion> {
    Handle::new(GeomSurfaceOfLinearExtrusion::new(&theCurve.cast(), theDirection).unwrap())
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__BasicEvaluation() {
    // Create a line along X axis
    let aLine = LineAlongX();

    // Extrude along Z axis -> Creates a plane
    let aDirection = dir(0.0, 0.0, 1.0);
    let anExtSurf = Extrusion(&aLine, aDirection);

    let anEval = GeomGridEvalSurfaceOfExtrusion::new(anExtSurf.clone());

    let aUParams = CreateUniformParams(0.0, 10.0, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Point3 = anExtSurf.value(aUParams[i], aVParams[j]).unwrap();
            let aPnt: Point3 = *aGrid.value(i, j).unwrap();
            EXPECT_NEAR(aPnt.distance(&aExpected), 0.0, THE_TOLERANCE);

            // Verify: S(u,v) = C(u) + v * Dir = (u, 0, 0) + v * (0, 0, 1) = (u, 0, v)
            EXPECT_NEAR(aPnt.x(), aUParams[i], THE_TOLERANCE);
            EXPECT_NEAR(aPnt.y(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aPnt.z(), aVParams[j], THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__CircleBasisCurve() {
    // Create a circle in XY plane with center at origin and radius 5
    let aCircle = CircleXY(5.0);

    // Extrude along Z axis -> Creates a cylinder
    let aDirection = dir(0.0, 0.0, 1.0);
    let anExtSurf = Extrusion(&aCircle, aDirection);

    let anEval = GeomGridEvalSurfaceOfExtrusion::new(anExtSurf.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 10.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=9 {
        for j in 1..=5 {
            let aExpected: Point3 = anExtSurf.value(aUParams[i], aVParams[j]).unwrap();
            let aPnt: Point3 = *aGrid.value(i, j).unwrap();
            EXPECT_NEAR(aPnt.distance(&aExpected), 0.0, THE_TOLERANCE);

            // Check cylindrical radius
            let aRadius: f64 = (aPnt.x() * aPnt.x() + aPnt.y() * aPnt.y()).sqrt();
            EXPECT_NEAR(aRadius, 5.0, THE_TOLERANCE);

            // Check Z coordinate equals V parameter
            EXPECT_NEAR(aPnt.z(), aVParams[j], THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__DerivativeD1() {
    let aLine = LineAlongX();
    let aDirection = dir(0.0, 0.0, 1.0);
    let anExtSurf = Extrusion(&aLine, aDirection);

    let anEval = GeomGridEvalSurfaceOfExtrusion::new(anExtSurf.clone());

    let aUParams = CreateUniformParams(0.0, 10.0, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGridD1: Array2<GeomGridEvalSurfD1> =
        anEval.evaluate_grid_d1(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = anExtSurf.d1(aUParams[i], aVParams[j]).unwrap();

            let aData = aGridD1.value(i, j).unwrap();
            EXPECT_NEAR(aData.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);

            // D1V should be the extrusion direction
            EXPECT_NEAR(aData.d1v.x(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aData.d1v.y(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aData.d1v.z(), 1.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__DerivativeD2() {
    // Use circle for non-trivial D2U
    let aCircle = CircleXY(5.0);
    let aDirection = dir(0.0, 0.0, 1.0);
    let anExtSurf = Extrusion(&aCircle, aDirection);

    let anEval = GeomGridEvalSurfaceOfExtrusion::new(anExtSurf.clone());

    let aUParams = CreateUniformParams(0.0, PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGridD2: Array2<GeomGridEvalSurfD2> =
        anEval.evaluate_grid_d2(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = anExtSurf.d2(aUParams[i], aVParams[j]).unwrap();

            let aData = aGridD2.value(i, j).unwrap();
            EXPECT_NEAR(aData.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);

            // D2V and D2UV should be zero for extrusion surface
            EXPECT_NEAR(aData.d2v.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aData.d2uv.magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__DerivativeD3() {
    // Use circle for non-trivial D3U
    let aCircle = CircleXY(5.0);
    let aDirection = dir(0.0, 0.0, 1.0);
    let anExtSurf = Extrusion(&aCircle, aDirection);

    let anEval = GeomGridEvalSurfaceOfExtrusion::new(anExtSurf.clone());

    let aUParams = CreateUniformParams(0.0, PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGridD3: Array2<GeomGridEvalSurfD3> =
        anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let r = anExtSurf.d3(aUParams[i], aVParams[j]).unwrap();

            let aData = aGridD3.value(i, j).unwrap();
            EXPECT_NEAR(aData.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3u - r.d3u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3v - r.d3v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3uuv - r.d3uuv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((aData.d3uvv - r.d3uvv).magnitude(), 0.0, THE_TOLERANCE);

            // D3V, D3UUV, D3UVV should all be zero for extrusion surface
            EXPECT_NEAR(aData.d3v.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aData.d3uuv.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aData.d3uvv.magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__UnifiedDispatch() {
    let aLine = LineAlongX();
    let aDirection = dir(0.0, 0.0, 1.0);
    let anExtSurf = Extrusion(&aLine, aDirection);

    // Test dispatch via unified evaluator
    let anEval = GeomGridEvalSurface::from_surface(&anExtSurf.cast());
    assert_eq!(anEval.get_type(), SurfaceType::SurfaceOfExtrusion);

    let aUParams = CreateUniformParams(0.0, 10.0, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Point3 = anExtSurf.value(aUParams[i], aVParams[j]).unwrap();
            EXPECT_NEAR(
                aGrid.value(i, j).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__AdaptorDispatch() {
    let aLine = LineAlongX();
    let aDirection = dir(0.0, 0.0, 1.0);
    let anExtSurf = Extrusion(&aLine, aDirection);

    // Test dispatch via adaptor
    let anAdaptor = GeomAdaptorSurface::with_surface(&anExtSurf.cast()).unwrap();
    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), SurfaceType::SurfaceOfExtrusion);

    let aUParams = CreateUniformParams(0.0, 10.0, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Point3 = anExtSurf.value(aUParams[i], aVParams[j]).unwrap();
            EXPECT_NEAR(
                aGrid.value(i, j).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_SurfaceOfExtrusionTest__NonAxisAlignedDirection() {
    // Create a circle and extrude along a diagonal direction
    let aCircle = CircleXY(3.0);

    // Extrude along (1, 1, 1) direction
    let aDirection = dir(1.0, 1.0, 1.0);
    let anExtSurf = Extrusion(&aCircle, aDirection);

    let anEval = GeomGridEvalSurfaceOfExtrusion::new(anExtSurf.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(-5.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=9 {
        for j in 1..=5 {
            let aExpected: Point3 = anExtSurf.value(aUParams[i], aVParams[j]).unwrap();
            EXPECT_NEAR(
                aGrid.value(i, j).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}
