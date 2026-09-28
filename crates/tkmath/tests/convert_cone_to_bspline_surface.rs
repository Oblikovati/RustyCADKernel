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

//! Direct translation of Convert_ConeToBSplineSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkmath::bsplslib::{self, SurfaceSpan};
use rustycad_tkmath::convert::ElementarySurfaceToBSplineSurface;
use rustycad_tkmath::gp::{Axis3, Cone3, Direction3, Point3};
use std::f64::consts::PI;

// gp_Cone(gp_Ax3(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), semiAngle, radius).
fn cone(semi_angle: f64, radius: f64) -> Cone3 {
    let frame = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Cone3::new(frame, semi_angle, radius).unwrap()
}

#[test]
fn Convert_ConeToBSplineSurfaceTest__FullCone() {
    let aCone = cone(PI / 6.0, 3.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_cone(&aCone, 0.0, 10.0).unwrap();

    assert_eq!(aConv.u_degree(), 2);
    assert_eq!(aConv.v_degree(), 1);
    assert!(aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
    assert!(aConv.nb_u_poles() > 0);
    assert!(aConv.nb_v_poles() > 0);
}

#[test]
fn Convert_ConeToBSplineSurfaceTest__TrimmedCone() {
    let aCone = cone(PI / 4.0, 2.0);
    let (aU1, aU2) = (0.0, PI);
    let (aV1, aV2) = (0.0, 5.0);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_cone_patch(&aCone, aU1, aU2, aV1, aV2).unwrap();

    assert!(!aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
}

#[test]
fn Convert_ConeToBSplineSurfaceTest__WeightsArePositive() {
    let aCone = cone(PI / 6.0, 1.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_cone(&aCone, 0.0, 5.0).unwrap();

    let aWeights = aConv.weights();
    for i in 1..=aConv.nb_u_poles() {
        for j in 1..=aConv.nb_v_poles() {
            assert!(aWeights[(i, j)] > 0.0);
        }
    }
}

#[test]
fn Convert_ConeToBSplineSurfaceTest__GeometricVerification() {
    let aSemiAngle: f64 = PI / 6.0;
    let aRadius: f64 = 3.0;
    let aCone = cone(aSemiAngle, aRadius);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_cone_patch(&aCone, 0.0, PI, 0.0, 10.0).unwrap();

    let aTol: f64 = 1.0e-10;
    let aUK = aConv.u_knots();
    let aVK = aConv.v_knots();
    let (aUMin, aUMax) = (aUK[aUK.lower()], aUK[aUK.upper()]);
    let (aVMin, aVMax) = (aVK[aVK.lower()], aVK[aVK.upper()]);

    for i in 0..=4 {
        let aU: f64 = aUMin + f64::from(i) * (aUMax - aUMin) / 4.0;
        for j in 0..=4 {
            let aV: f64 = aVMin + f64::from(j) * (aVMax - aVMin) / 4.0;

            let aSpan = SurfaceSpan {
                u: aU,
                v: aV,
                u_index: 0,
                v_index: 0,
                u_knots: aConv.u_knots(),
                v_knots: aConv.v_knots(),
                u_mults: Some(aConv.u_multiplicities()),
                v_mults: Some(aConv.v_multiplicities()),
                u_degree: aConv.u_degree(),
                v_degree: aConv.v_degree(),
                u_rat: true,
                v_rat: false,
                u_per: false,
                v_per: false,
            };
            let aPnt: Point3 = bsplslib::d0(&aSpan, aConv.poles(), Some(aConv.weights())).unwrap();

            // Verify the point lies on the cone:
            // For a cone with axis Z, semi-angle a, reference radius R:
            // r(z) = R + z * tan(a), distance from axis = r
            let aDistFromAxis: f64 = (aPnt.x() * aPnt.x() + aPnt.y() * aPnt.y()).sqrt();
            let aExpectedR: f64 = aRadius + aPnt.z() * aSemiAngle.tan();
            assert!(
                (aDistFromAxis - aExpectedR).abs() <= aTol,
                "Point not on cone at U={aU} V={aV}"
            );
        }
    }
}
