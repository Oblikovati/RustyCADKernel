// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Direct translation of Geom2dAPI_Interpolate_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::geom2d_api::Interpolate;
use rustycad_tkmath::gp::{Point2, Vector2};

#[test]
fn Geom2dAPI_InterpolateTest__OCC28594_InterpolateWithAndWithoutTangentScale() {
    let aPoints: Handle<HArray1<Point2>> = Handle::new(HArray1::with_bounds(1, 6).unwrap());
    {
        let mut aPointsArray = aPoints.get().unwrap().change_array1();
        aPointsArray[1] = Point2::new(-30.4, 8.0);
        aPointsArray[2] = Point2::new(-16.689912, 17.498217);
        aPointsArray[3] = Point2::new(-23.803064, 24.748543);
        aPointsArray[4] = Point2::new(-16.907466, 32.919615);
        aPointsArray[5] = Point2::new(-8.543829, 26.549421);
        aPointsArray[6] = Point2::new(0.0, 39.200000);
    }

    let mut aTangents: Array1<'static, Vector2> = Array1::with_bounds(1, 6).unwrap();
    aTangents[1] = Vector2::new(0.3, 0.4);
    aTangents[2] = Vector2::new(0.0, 0.0);
    aTangents[3] = Vector2::new(0.0, 0.0);
    aTangents[4] = Vector2::new(0.0, 0.0);
    aTangents[5] = Vector2::new(0.0, 0.0);
    aTangents[6] = Vector2::new(1.0, 0.0);

    let aTangentFlags: Handle<HArray1<bool>> = Handle::new(HArray1::with_bounds(1, 6).unwrap());
    {
        let mut aTangentFlagsArray = aTangentFlags.get().unwrap().change_array1();
        aTangentFlagsArray[1] = true;
        aTangentFlagsArray[2] = false;
        aTangentFlagsArray[3] = false;
        aTangentFlagsArray[4] = false;
        aTangentFlagsArray[5] = false;
        aTangentFlagsArray[6] = true;
    }

    // Interpolation with tangent scale
    let mut anInterpWithScale = Interpolate::new(&aPoints, false, precision::confusion()).unwrap();
    anInterpWithScale
        .load(&aTangents, &aTangentFlags, true)
        .unwrap();
    anInterpWithScale.perform().unwrap();
    assert!(anInterpWithScale.is_done());
    let aCurveWithScale = anInterpWithScale.curve().unwrap();
    assert!(!aCurveWithScale.is_null());

    // Interpolation without tangent scale
    let mut anInterpWithoutScale =
        Interpolate::new(&aPoints, false, precision::confusion()).unwrap();
    anInterpWithoutScale
        .load(&aTangents, &aTangentFlags, false)
        .unwrap();
    anInterpWithoutScale.perform().unwrap();
    assert!(anInterpWithoutScale.is_done());
    let aCurveWithoutScale = anInterpWithoutScale.curve().unwrap();
    assert!(!aCurveWithoutScale.is_null());

    // Both curves must pass through all given points
    let aTol = precision::confusion() * 10.0;
    let aLength = aPoints.get().unwrap().array1().len() as i32;
    for anIndex in 1..=aLength {
        let curve = aCurveWithScale.get().unwrap();
        let aPtOnCurveWithScale = curve.eval_d0(curve.knot(anIndex).unwrap()).unwrap();
        let aPt = aPoints.get().unwrap().array1()[anIndex];
        EXPECT_NEAR(aPt.x(), aPtOnCurveWithScale.x(), aTol);
        EXPECT_NEAR(aPt.y(), aPtOnCurveWithScale.y(), aTol);
    }
}
