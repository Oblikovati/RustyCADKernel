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

//! Direct translation of Convert_GridPolynomialToPoles_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::convert::GridPolynomialToPoles;

// NCollection_Array1<T>(1, n).
fn array<T: Default>(count: i32) -> Array1<'static, T> {
    Array1::with_bounds(1, count).unwrap()
}

#[test]
fn Convert_GridPolynomialToPolesTest__SinglePlanarPatch() {
    // A single bilinear patch: z = 0, x = u, y = v
    // MaxUDegree = 1, MaxVDegree = 1
    // Coefficients as "C array" [MaxUDegree+1][MaxVDegree+1][3]
    // For P(u,v) = (u, v, 0): coefficients in polynomial form
    let aMaxUDeg: i32 = 1;
    let aMaxVDeg: i32 = 1;

    let mut aNumCoeff: Array1<'static, i32> = array(2);
    aNumCoeff.set_value(1, 2).unwrap(); // U degree + 1
    aNumCoeff.set_value(2, 2).unwrap(); // V degree + 1

    // Coefficients [2][2][3] = 12 values
    // Layout: for u_i, v_j: coeff(u_i, v_j) = {x, y, z}
    // P(u,v) = sum c_{ij} * u^i * v^j
    // c_{00} = (0,0,0), c_{10} = (1,0,0), c_{01} = (0,1,0), c_{11} = (0,0,0)
    let mut aCoeffs: Array1<'static, f64> = array(12);
    // c_{00}: x=0,y=0,z=0
    aCoeffs.set_value(1, 0.0).unwrap();
    aCoeffs.set_value(2, 0.0).unwrap();
    aCoeffs.set_value(3, 0.0).unwrap();
    // c_{01}: x=0,y=1,z=0
    aCoeffs.set_value(4, 0.0).unwrap();
    aCoeffs.set_value(5, 1.0).unwrap();
    aCoeffs.set_value(6, 0.0).unwrap();
    // c_{10}: x=1,y=0,z=0
    aCoeffs.set_value(7, 1.0).unwrap();
    aCoeffs.set_value(8, 0.0).unwrap();
    aCoeffs.set_value(9, 0.0).unwrap();
    // c_{11}: x=0,y=0,z=0
    aCoeffs.set_value(10, 0.0).unwrap();
    aCoeffs.set_value(11, 0.0).unwrap();
    aCoeffs.set_value(12, 0.0).unwrap();

    let mut aPolyU: Array1<'static, f64> = array(2);
    aPolyU.set_value(1, 0.0).unwrap();
    aPolyU.set_value(2, 1.0).unwrap();

    let mut aPolyV: Array1<'static, f64> = array(2);
    aPolyV.set_value(1, 0.0).unwrap();
    aPolyV.set_value(2, 1.0).unwrap();

    let aConv = GridPolynomialToPoles::from_single_patch(
        aMaxUDeg, aMaxVDeg, &aNumCoeff, &aCoeffs, &aPolyU, &aPolyV,
    )
    .unwrap();

    assert!(aConv.is_done());
    assert!(aConv.nb_u_poles().unwrap() > 0);
    assert!(aConv.nb_v_poles().unwrap() > 0);
    assert_eq!(aConv.u_degree().unwrap(), 1);
    assert_eq!(aConv.v_degree().unwrap(), 1);
    assert!(aConv.nb_u_knots().unwrap() > 0);
    assert!(aConv.nb_v_knots().unwrap() > 0);

    let aPoles = aConv.poles().unwrap();
    assert!(!aPoles.is_empty());
}

#[test]
fn Convert_GridPolynomialToPolesTest__QueryMethods() {
    let aMaxUDeg: i32 = 1;
    let aMaxVDeg: i32 = 1;

    let mut aNumCoeff: Array1<'static, i32> = array(2);
    aNumCoeff.set_value(1, 2).unwrap();
    aNumCoeff.set_value(2, 2).unwrap();

    let mut aCoeffs: Array1<'static, f64> = array(12);
    for i in 1..=12 {
        aCoeffs.set_value(i, 0.0).unwrap();
    }
    // Just set x = u coefficient
    aCoeffs.set_value(7, 1.0).unwrap();
    // y = v coefficient
    aCoeffs.set_value(5, 1.0).unwrap();

    let mut aPolyU: Array1<'static, f64> = array(2);
    aPolyU.set_value(1, 0.0).unwrap();
    aPolyU.set_value(2, 1.0).unwrap();

    let mut aPolyV: Array1<'static, f64> = array(2);
    aPolyV.set_value(1, 0.0).unwrap();
    aPolyV.set_value(2, 1.0).unwrap();

    let aConv = GridPolynomialToPoles::from_single_patch(
        aMaxUDeg, aMaxVDeg, &aNumCoeff, &aCoeffs, &aPolyU, &aPolyV,
    )
    .unwrap();

    assert!(aConv.is_done());

    let aUKnots = aConv.u_knots().unwrap();
    let aVKnots = aConv.v_knots().unwrap();
    let aUMults = aConv.u_multiplicities().unwrap();
    let aVMults = aConv.v_multiplicities().unwrap();

    assert!(!aUKnots.is_empty());
    assert!(!aVKnots.is_empty());
    assert!(!aUMults.is_empty());
    assert!(!aVMults.is_empty());

    assert_eq!(aUKnots.len() as i32, aConv.nb_u_knots().unwrap());
    assert_eq!(aVKnots.len() as i32, aConv.nb_v_knots().unwrap());
}
