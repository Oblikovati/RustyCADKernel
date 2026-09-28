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

//! Direct translation of Convert_TorusToBSplineSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkmath::bsplslib::{self, SurfaceSpan};
use rustycad_tkmath::convert::ElementarySurfaceToBSplineSurface;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Torus3};
use std::f64::consts::PI;

// gp_Ax3(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)).
fn frame() -> Axis3 {
    Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

fn torus(major: f64, minor: f64) -> Torus3 {
    Torus3::new(frame(), major, minor).unwrap()
}

#[test]
fn Convert_TorusToBSplineSurfaceTest__FullTorus() {
    let aTorus = torus(5.0, 2.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_torus(&aTorus).unwrap();

    assert_eq!(aConv.u_degree(), 2);
    assert_eq!(aConv.v_degree(), 2);
    assert!(aConv.is_u_periodic());
    assert!(aConv.is_v_periodic());
    assert!(aConv.nb_u_poles() > 0);
    assert!(aConv.nb_v_poles() > 0);
}

#[test]
fn Convert_TorusToBSplineSurfaceTest__TrimmedUV() {
    let aTorus = torus(5.0, 2.0);
    let (aU1, aU2) = (0.0, PI);
    let (aV1, aV2) = (0.0, PI);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_torus_patch(&aTorus, aU1, aU2, aV1, aV2).unwrap();

    assert!(!aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
    assert!(aConv.nb_u_poles() > 0);
    assert!(aConv.nb_v_poles() > 0);
}

#[test]
fn Convert_TorusToBSplineSurfaceTest__UTrimmed() {
    let aTorus = torus(5.0, 2.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_torus_band(&aTorus, 0.0, PI, true).unwrap();

    assert!(!aConv.is_u_periodic());
    assert!(aConv.is_v_periodic());
}

#[test]
fn Convert_TorusToBSplineSurfaceTest__VTrimmed() {
    let aTorus = torus(5.0, 2.0);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_torus_band(&aTorus, 0.0, PI, false).unwrap();

    assert!(aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
}

#[test]
fn Convert_TorusToBSplineSurfaceTest__WeightsArePositive() {
    let aTorus = torus(5.0, 2.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_torus(&aTorus).unwrap();

    let aWeights = aConv.weights();
    for i in 1..=aConv.nb_u_poles() {
        for j in 1..=aConv.nb_v_poles() {
            assert!(aWeights[(i, j)] > 0.0);
        }
    }
}

#[test]
fn Convert_TorusToBSplineSurfaceTest__KnotsAreMonotonic() {
    let aTorus = torus(5.0, 2.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_torus(&aTorus).unwrap();

    let aUKnots = aConv.u_knots();
    for i in 2..=aConv.nb_u_knots() {
        assert!(aUKnots[i] > aUKnots[i - 1]);
    }
    let aVKnots = aConv.v_knots();
    for i in 2..=aConv.nb_v_knots() {
        assert!(aVKnots[i] > aVKnots[i - 1]);
    }
}

#[test]
fn Convert_TorusToBSplineSurfaceTest__GeometricVerification() {
    let aMajorRadius: f64 = 5.0;
    let aMinorRadius: f64 = 2.0;
    let aTorus = Torus3::new(frame(), aMajorRadius, aMinorRadius).unwrap();
    let aConv =
        ElementarySurfaceToBSplineSurface::from_torus_patch(&aTorus, 0.0, PI, 0.0, PI).unwrap();

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
                v_rat: true,
                u_per: false,
                v_per: false,
            };
            let aPnt: Point3 = bsplslib::d0(&aSpan, aConv.poles(), Some(aConv.weights())).unwrap();

            // Verify the point lies on the torus:
            // (sqrt(x^2 + y^2) - R_major)^2 + z^2 = R_minor^2
            let aRxy: f64 = (aPnt.x() * aPnt.x() + aPnt.y() * aPnt.y()).sqrt();
            let aDistSq: f64 = (aRxy - aMajorRadius) * (aRxy - aMajorRadius) + aPnt.z() * aPnt.z();
            assert!(
                (aDistSq - aMinorRadius * aMinorRadius).abs() <= aTol,
                "Point not on torus at U={aU} V={aV}"
            );
        }
    }
}
