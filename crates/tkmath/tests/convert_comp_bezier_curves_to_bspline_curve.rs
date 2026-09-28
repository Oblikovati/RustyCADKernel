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

//! Direct translation of Convert_CompBezierCurvesToBSplineCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::convert::{
    CompBezierCurves2dToBSplineCurve2d, CompBezierCurvesToBSplineCurve,
};
use rustycad_tkmath::gp::{Point2, Point3};

// NCollection_Array1<T>(lower, upper).
fn array<T: Default>(lower: i32, upper: i32) -> Array1<'static, T> {
    Array1::with_bounds(lower, upper).unwrap()
}

#[test]
fn Convert_CompBezierCurvesToBSplineCurveTest__SingleLinearBezier() {
    let mut aConv = CompBezierCurvesToBSplineCurve::default();

    let mut aPoles: Array1<'static, Point3> = array(1, 2);
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 1.0, 0.0);

    aConv.add_curve(&aPoles);
    aConv.perform().unwrap();

    assert!(aConv.degree() >= 1);
    assert_eq!(aConv.nb_poles(), 2);
    assert_eq!(aConv.nb_knots(), 2);

    let mut aResPoles: Array1<'static, Point3> = array(1, aConv.nb_poles());
    aConv.poles(&mut aResPoles).unwrap();
    assert!((aResPoles[1].x() - 0.0).abs() <= 1.0e-15);
    assert!((aResPoles[2].x() - 1.0).abs() <= 1.0e-15);
}

#[test]
fn Convert_CompBezierCurvesToBSplineCurveTest__SingleCubicBezier() {
    let mut aConv = CompBezierCurvesToBSplineCurve::default();

    let mut aPoles: Array1<'static, Point3> = array(1, 4);
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(3.0, 2.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);

    aConv.add_curve(&aPoles);
    aConv.perform().unwrap();

    assert_eq!(aConv.degree(), 3);
    assert_eq!(aConv.nb_poles(), 4);
    assert_eq!(aConv.nb_knots(), 2);
}

#[test]
fn Convert_CompBezierCurvesToBSplineCurveTest__TwoAdjacentBeziers_C0() {
    let mut aConv = CompBezierCurvesToBSplineCurve::default();

    // First linear segment
    let mut aPoles1: Array1<'static, Point3> = array(1, 2);
    aPoles1[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles1[2] = Point3::new(1.0, 1.0, 0.0);

    // Second linear segment, adjacent but not tangent
    let mut aPoles2: Array1<'static, Point3> = array(1, 2);
    aPoles2[1] = Point3::new(1.0, 1.0, 0.0);
    aPoles2[2] = Point3::new(2.0, 0.0, 0.0);

    aConv.add_curve(&aPoles1);
    aConv.add_curve(&aPoles2);
    aConv.perform().unwrap();

    assert!(aConv.degree() >= 1);
    assert_eq!(aConv.nb_knots(), 3);

    let mut aResPoles: Array1<'static, Point3> = array(1, aConv.nb_poles());
    let mut aKnots: Array1<'static, f64> = array(1, aConv.nb_knots());
    let mut aMults: Array1<'static, i32> = array(1, aConv.nb_knots());
    aConv.poles(&mut aResPoles).unwrap();
    aConv.knots_and_mults(&mut aKnots, &mut aMults).unwrap();

    // First and last knot multiplicities should be degree+1
    assert_eq!(aMults[1], aConv.degree() + 1);
    assert_eq!(aMults[aConv.nb_knots()], aConv.degree() + 1);
}

#[test]
fn Convert_CompBezierCurvesToBSplineCurveTest__TwoAdjacentBeziers_C1() {
    let mut aConv = CompBezierCurvesToBSplineCurve::default();

    // Two cubic Beziers with parallel tangent at junction
    let mut aPoles1: Array1<'static, Point3> = array(1, 4);
    aPoles1[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles1[2] = Point3::new(1.0, 1.0, 0.0);
    aPoles1[3] = Point3::new(2.0, 1.0, 0.0);
    aPoles1[4] = Point3::new(3.0, 0.0, 0.0);

    let mut aPoles2: Array1<'static, Point3> = array(1, 4);
    aPoles2[1] = Point3::new(3.0, 0.0, 0.0);
    aPoles2[2] = Point3::new(4.0, -1.0, 0.0);
    aPoles2[3] = Point3::new(5.0, -1.0, 0.0);
    aPoles2[4] = Point3::new(6.0, 0.0, 0.0);

    aConv.add_curve(&aPoles1);
    aConv.add_curve(&aPoles2);
    aConv.perform().unwrap();

    assert_eq!(aConv.degree(), 3);
    assert_eq!(aConv.nb_knots(), 3);

    // Check that junction has multiplicity Degree-1 (C1)
    let mut aMults: Array1<'static, i32> = array(1, aConv.nb_knots());
    let mut aKnots: Array1<'static, f64> = array(1, aConv.nb_knots());
    aConv.knots_and_mults(&mut aKnots, &mut aMults).unwrap();
    assert_eq!(aMults[2], aConv.degree() - 1);
}

#[test]
fn Convert_CompBezierCurvesToBSplineCurveTest__MixedDegreeBeziers() {
    let mut aConv = CompBezierCurvesToBSplineCurve::default();

    // Linear segment
    let mut aPoles1: Array1<'static, Point3> = array(1, 2);
    aPoles1[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles1[2] = Point3::new(1.0, 0.0, 0.0);

    // Cubic segment
    let mut aPoles2: Array1<'static, Point3> = array(1, 4);
    aPoles2[1] = Point3::new(1.0, 0.0, 0.0);
    aPoles2[2] = Point3::new(2.0, 1.0, 0.0);
    aPoles2[3] = Point3::new(3.0, 1.0, 0.0);
    aPoles2[4] = Point3::new(4.0, 0.0, 0.0);

    aConv.add_curve(&aPoles1);
    aConv.add_curve(&aPoles2);
    aConv.perform().unwrap();

    // Degree should be raised to the maximum
    assert_eq!(aConv.degree(), 3);
}

// 2D variant tests
#[test]
fn Convert_CompBezierCurves2dToBSplineCurve2dTest__SingleLinear2d() {
    let mut aConv = CompBezierCurves2dToBSplineCurve2d::default();

    let mut aPoles: Array1<'static, Point2> = array(1, 2);
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);

    aConv.add_curve(&aPoles);
    aConv.perform().unwrap();

    assert!(aConv.degree() >= 1);
    assert_eq!(aConv.nb_poles(), 2);
    assert_eq!(aConv.nb_knots(), 2);

    let mut aResPoles: Array1<'static, Point2> = array(1, aConv.nb_poles());
    aConv.poles(&mut aResPoles).unwrap();
    assert!((aResPoles[1].x() - 0.0).abs() <= 1.0e-15);
    assert!((aResPoles[2].x() - 1.0).abs() <= 1.0e-15);
}

#[test]
fn Convert_CompBezierCurves2dToBSplineCurve2dTest__TwoAdjacent2d_C1() {
    let mut aConv = CompBezierCurves2dToBSplineCurve2d::default();

    let mut aPoles1: Array1<'static, Point2> = array(1, 4);
    aPoles1[1] = Point2::new(0.0, 0.0);
    aPoles1[2] = Point2::new(1.0, 1.0);
    aPoles1[3] = Point2::new(2.0, 1.0);
    aPoles1[4] = Point2::new(3.0, 0.0);

    let mut aPoles2: Array1<'static, Point2> = array(1, 4);
    aPoles2[1] = Point2::new(3.0, 0.0);
    aPoles2[2] = Point2::new(4.0, -1.0);
    aPoles2[3] = Point2::new(5.0, -1.0);
    aPoles2[4] = Point2::new(6.0, 0.0);

    aConv.add_curve(&aPoles1);
    aConv.add_curve(&aPoles2);
    aConv.perform().unwrap();

    assert_eq!(aConv.degree(), 3);
    assert_eq!(aConv.nb_knots(), 3);

    let mut aMults: Array1<'static, i32> = array(1, aConv.nb_knots());
    let mut aKnots: Array1<'static, f64> = array(1, aConv.nb_knots());
    aConv.knots_and_mults(&mut aKnots, &mut aMults).unwrap();
    assert_eq!(aMults[2], aConv.degree() - 1);
}
