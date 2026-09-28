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

//! Direct translation of GeomGridEval_OffsetSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomOffsetSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalOffsetSurface, GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3,
    GeomGridEvalSurface,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-9;

fn ZDir() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}

// `new Geom_Plane(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1))`.
fn PlaneXY() -> Handle<GeomPlane> {
    Handle::new(GeomPlane::from_point(Point3::new(0.0, 0.0, 0.0), ZDir()).unwrap())
}

// `new Geom_CylindricalSurface(gp_Ax3(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 5.0)`.
fn Cylinder5() -> Handle<GeomCylindricalSurface> {
    let anAxis = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), ZDir()).unwrap();
    Handle::new(GeomCylindricalSurface::new(anAxis, 5.0).unwrap())
}

// `new Geom_OffsetSurface(theBasis, theOffset)`.
fn Offset<S: RuntimeTyped + ?Sized>(
    theBasis: &Handle<S>,
    theOffset: f64,
) -> Handle<GeomOffsetSurface> {
    Handle::new(GeomOffsetSurface::new(&theBasis.cast(), theOffset, false).unwrap())
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__PlaneOffset() {
    // Plane at Z=0
    let aPlane = PlaneXY();
    // Offset by 10.0 -> Plane at Z=10
    let anOffset = Offset(&aPlane, 10.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    let aUParams = CreateUniformParams(0.0, 10.0, 5);
    let aVParams = CreateUniformParams(0.0, 10.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpected: Point3 = anOffset.value(aUParams[i], aVParams[j]).unwrap();
            EXPECT_NEAR(
                aGrid.value(i, j).unwrap().distance(&aExpected),
                0.0,
                THE_TOLERANCE,
            );
            EXPECT_NEAR(aGrid.value(i, j).unwrap().z(), 10.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__CylinderOffset() {
    // Cylinder radius 5
    let aCyl = Cylinder5();
    // Offset by 2.0 -> Cylinder radius 7 (outward)
    let anOffset = Offset(&aCyl, 2.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for i in 1..=9 {
        for j in 1..=5 {
            let aExpected: Point3 = anOffset.value(aUParams[i], aVParams[j]).unwrap();
            let aPnt: Point3 = *aGrid.value(i, j).unwrap();
            EXPECT_NEAR(aPnt.distance(&aExpected), 0.0, THE_TOLERANCE);

            // Check radius from Z axis
            let aDist: f64 = (aPnt.x() * aPnt.x() + aPnt.y() * aPnt.y()).sqrt();
            EXPECT_NEAR(aDist, 7.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__DerivativeD1() {
    // Cylinder offset
    let aCyl = Cylinder5();
    let anOffset = Offset(&aCyl, 2.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 3);

    let aGrid: Array2<GeomGridEvalSurfD1> = anEval.evaluate_grid_d1(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=3 {
            let r = anOffset.d1(aUParams[i], aVParams[j]).unwrap();
            let g = aGrid.value(i, j).unwrap();
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__NestedDispatch() {
    // Test that GeomGridEval_Surface correctly dispatches to GeomGridEval_OffsetSurface
    let aPlane = PlaneXY();
    let anOffset = Offset(&aPlane, 5.0);

    let anEval = GeomGridEvalSurface::from_surface(&anOffset.cast());
    // Assuming GetType() returns OffsetSurface or similar identifier
    // (GeomGridEval_Surface::GetType logic might need checking, usually delegates to adaptor or
    // stored type) Since we added OffsetSurface to variant, it should be supported.

    let aUParams = CreateUniformParams(0.0, 1.0, 3);
    let aVParams = CreateUniformParams(0.0, 1.0, 3);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();
    // RowLength() is the number of columns, ColLength() the number of rows.
    assert_eq!(aGrid.nb_columns(), 3);
    assert_eq!(aGrid.nb_rows(), 3);
    EXPECT_NEAR(aGrid.value(1, 1).unwrap().z(), 5.0, THE_TOLERANCE);
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__DerivativeD2() {
    // Cylinder offset
    let aCyl = Cylinder5();
    let anOffset = Offset(&aCyl, 2.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 3);

    let aGrid: Array2<GeomGridEvalSurfD2> = anEval.evaluate_grid_d2(&aUParams, &aVParams).unwrap();

    for i in 1..=5 {
        for j in 1..=3 {
            let r = anOffset.d2(aUParams[i], aVParams[j]).unwrap();
            let g = aGrid.value(i, j).unwrap();
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
fn GeomGridEval_OffsetSurfaceTest__DerivativeD3() {
    // Cylinder offset
    let aCyl = Cylinder5();
    let anOffset = Offset(&aCyl, 2.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 3);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    let anAdaptor = GeomAdaptorSurface::with_surface(&anOffset.cast()).unwrap();

    for i in 1..=5 {
        for j in 1..=3 {
            let r = anAdaptor.d3(aUParams[i], aVParams[j]).unwrap();
            let g = aGrid.value(i, j).unwrap();
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

// The body shared by the four cylinder-offset DN declarations: the grid DN of
// order `(theNU, theNV)` against `Geom_OffsetSurface::DN`.
fn CheckCylinderOffsetDN(theNU: i32, theNV: i32) {
    // Cylinder offset
    let aCyl = Cylinder5();
    let anOffset = Offset(&aCyl, 2.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 5);
    let aVParams = CreateUniformParams(0.0, 5.0, 3);

    let aGrid: Array2<Vector3> = anEval
        .evaluate_grid_dn(&aUParams, &aVParams, theNU, theNV)
        .unwrap();

    for i in 1..=5 {
        for j in 1..=3 {
            let aExpected: Vector3 = anOffset.dn(aUParams[i], aVParams[j], theNU, theNV).unwrap();
            EXPECT_NEAR(
                (*aGrid.value(i, j).unwrap() - aExpected).magnitude(),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__DerivativeDN_U1V0() {
    // Use Geom_OffsetSurface::DN as reference (GeomAdaptor_Surface::DN doesn't
    // properly handle offset surfaces - it returns base surface derivatives)
    CheckCylinderOffsetDN(1, 0);
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__DerivativeDN_U0V1() {
    CheckCylinderOffsetDN(0, 1);
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__DerivativeDN_U1V1() {
    CheckCylinderOffsetDN(1, 1);
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__DerivativeDN_U2V0() {
    CheckCylinderOffsetDN(2, 0);
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__DerivativeDN_PlaneOffset() {
    // Plane offset - simpler case for testing
    let aPlane = PlaneXY();
    let anOffset = Offset(&aPlane, 10.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    let aUParams = CreateUniformParams(0.0, 10.0, 5);
    let aVParams = CreateUniformParams(0.0, 10.0, 5);

    // Test DN(1,0) and DN(0,1) - use Geom_OffsetSurface::DN as reference
    let aGridU: Array2<Vector3> = anEval.evaluate_grid_dn(&aUParams, &aVParams, 1, 0).unwrap();
    let aGridV: Array2<Vector3> = anEval.evaluate_grid_dn(&aUParams, &aVParams, 0, 1).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let aExpectedU: Vector3 = anOffset.dn(aUParams[i], aVParams[j], 1, 0).unwrap();
            let aExpectedV: Vector3 = anOffset.dn(aUParams[i], aVParams[j], 0, 1).unwrap();
            EXPECT_NEAR(
                (*aGridU.value(i, j).unwrap() - aExpectedU).magnitude(),
                0.0,
                THE_TOLERANCE,
            );
            EXPECT_NEAR(
                (*aGridV.value(i, j).unwrap() - aExpectedV).magnitude(),
                0.0,
                THE_TOLERANCE,
            );
        }
    }

    // For a plane offset, 2nd and higher derivatives should be zero
    let aGrid2U: Array2<Vector3> = anEval.evaluate_grid_dn(&aUParams, &aVParams, 2, 0).unwrap();
    for i in 1..=5 {
        for j in 1..=5 {
            EXPECT_NEAR(aGrid2U.value(i, j).unwrap().magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

// A one-element parameter array `aParams(1, 1)` holding `theValue`.
fn SingleParam(theValue: f64) -> Array1<'static, f64> {
    let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, 1).unwrap();
    aParams[1] = theValue;
    aParams
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__IsolineU_CompareToGeomD0() {
    // Cylinder offset
    let aCyl = Cylinder5();
    let anOffset = Offset(&aCyl, 2.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    // U-isoline: 1 U param, multiple V params (triggers isoline path)
    let aUParams = SingleParam(PI / 4.0);
    let aVParams = CreateUniformParams(0.0, 5.0, 15);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Note: RowLength() = V count (columns), ColLength() = U count (rows)
    assert_eq!(aGrid.nb_columns(), 15);
    assert_eq!(aGrid.nb_rows(), 1);

    // Compare against Geom_OffsetSurface::D0
    for j in 1..=15 {
        let aExpected: Point3 = anOffset.d0(aUParams[1], aVParams[j]).unwrap();
        EXPECT_NEAR(
            aGrid.value(1, j).unwrap().distance(&aExpected),
            0.0,
            THE_TOLERANCE,
        );
    }
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__IsolineV_CompareToGeomD0() {
    // Cylinder offset
    let aCyl = Cylinder5();
    let anOffset = Offset(&aCyl, 2.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    // V-isoline: multiple U params, 1 V param (triggers isoline path)
    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 15);
    let aVParams = SingleParam(2.5);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    // Note: RowLength() = V count (columns), ColLength() = U count (rows)
    assert_eq!(aGrid.nb_columns(), 1);
    assert_eq!(aGrid.nb_rows(), 15);

    // Compare against Geom_OffsetSurface::D0
    for i in 1..=15 {
        let aExpected: Point3 = anOffset.d0(aUParams[i], aVParams[1]).unwrap();
        EXPECT_NEAR(
            aGrid.value(i, 1).unwrap().distance(&aExpected),
            0.0,
            THE_TOLERANCE,
        );
    }
}

#[test]
fn GeomGridEval_OffsetSurfaceTest__IsolinePlane_CompareToGeomD0() {
    // Plane offset (simple case)
    let aPlane = PlaneXY();
    let anOffset = Offset(&aPlane, 3.0);

    let anEval = GeomGridEvalOffsetSurface::new(anOffset.clone());

    // U-isoline on plane offset
    let aUParams = SingleParam(5.0);
    let aVParams = CreateUniformParams(-10.0, 10.0, 20);

    let aGrid: Array2<Point3> = anEval.evaluate_grid(&aUParams, &aVParams).unwrap();

    for j in 1..=20 {
        let aExpected: Point3 = anOffset.d0(aUParams[1], aVParams[j]).unwrap();
        EXPECT_NEAR(
            aGrid.value(1, j).unwrap().distance(&aExpected),
            0.0,
            THE_TOLERANCE,
        );
    }
}
