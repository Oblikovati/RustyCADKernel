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

//! Direct translation of GCPnts_QuasiUniformDeflection_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::prelude::*;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection as GCPnts_QuasiUniformDeflection;
use rustycad_tkmath::geom_abs::Shape;

fn makeOCC33828Curve() -> Handle<GeomBSplineCurve> {
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 7).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 0.17;
    aKnots[3] = 0.33;
    aKnots[4] = 0.5;
    aKnots[5] = 0.67;
    aKnots[6] = 0.83;
    aKnots[7] = 1.0;

    let mut aMultiplicities: Array1<'static, i32> = Array1::with_bounds(1, 7).unwrap();
    aMultiplicities[1] = 4;
    aMultiplicities[2] = 2;
    aMultiplicities[3] = 2;
    aMultiplicities[4] = 2;
    aMultiplicities[5] = 2;
    aMultiplicities[6] = 2;
    aMultiplicities[7] = 4;

    let aPoleCoordinates: [[f64; 3]; 14] = [
        [0.163, 0.233, 0.0],
        [0.158, 0.204, 0.0],
        [0.139, 0.180, 0.0],
        [0.086, 0.159, 0.0],
        [0.055, 0.163, 0.0],
        [0.009, 0.196, 0.0],
        [-0.004, 0.225, 0.0],
        [0.002, 0.281, 0.0],
        [0.019, 0.307, 0.0],
        [0.070, 0.332, 0.0],
        [0.101, 0.331, 0.0],
        [0.149, 0.301, 0.0],
        [0.164, 0.274, 0.0],
        [0.163, 0.246, 0.0],
    ];

    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 14).unwrap();
    for anIndex in 0..14 {
        aPoles[anIndex + 1] = Point3::new(
            aPoleCoordinates[anIndex as usize][0],
            aPoleCoordinates[anIndex as usize][1],
            aPoleCoordinates[anIndex as usize][2],
        );
    }

    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMultiplicities, 3, false).unwrap())
}

fn checkOCC33828Deflection(
    theCurve: &GeomAdaptorCurve,
    theDeflection: f64,
    theExpectedNbPoints: i32,
) {
    let anAlgo =
        GCPnts_QuasiUniformDeflection::with_curve(theCurve, theDeflection, Shape::C1).unwrap();
    assert!(anAlgo.is_done());
    assert_eq!(anAlgo.nb_points().unwrap(), theExpectedNbPoints);
    EXPECT_LE(anAlgo.deflection().unwrap(), theDeflection);
}

// Migrated from tests/bugs/moddata_3/bug33828. Small changes in requested
// deflection must produce the stable point counts recorded by the DRAW test.
#[test]
fn GCPnts_QuasiUniformDeflectionTest__ModDataBug_33828_StablePointCounts() {
    let aCurve: Handle<GeomBSplineCurve> = makeOCC33828Curve();
    let anAdaptor = GeomAdaptorCurve::with_curve(&aCurve.cast()).unwrap();

    checkOCC33828Deflection(&anAdaptor, 0.5, 2);
    checkOCC33828Deflection(&anAdaptor, 0.1, 3);
    checkOCC33828Deflection(&anAdaptor, 0.05, 5);
    checkOCC33828Deflection(&anAdaptor, 0.025, 5);
    checkOCC33828Deflection(&anAdaptor, 0.007, 9);
    checkOCC33828Deflection(&anAdaptor, 0.005, 17);
    checkOCC33828Deflection(&anAdaptor, 0.0005, 33);
    checkOCC33828Deflection(&anAdaptor, 0.0003, 65);
    checkOCC33828Deflection(&anAdaptor, 0.0002, 65);
    checkOCC33828Deflection(&anAdaptor, 0.0001, 73);
}
