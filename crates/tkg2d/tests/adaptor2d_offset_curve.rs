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

//! Direct translation of Adaptor2d_OffsetCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dLine2d, Adaptor2dOffsetCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Direction2, Point2};

fn dir(x: f64, y: f64) -> Direction2 {
    Direction2::new(x, y).unwrap()
}

// Adaptor2d_OffsetCurveTest::SetUp: horizontal line through origin, bounds [0, 10].
fn setup() -> Handle<dyn Adaptor2dCurve2d> {
    Handle::new(Adaptor2dLine2d::new(
        Point2::new(0.0, 0.0),
        dir(1.0, 0.0),
        0.0,
        10.0,
    ))
    .cast()
}

#[test]
fn Adaptor2d_OffsetCurveTest__DefaultConstructor() {
    let _myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::default();
    EXPECT_DOUBLE_EQ(anOffset.offset(), 0.0);
}

#[test]
fn Adaptor2d_OffsetCurveTest__ConstructWithCurve_OffsetIsZero() {
    let myBaseLine = setup();
    // Constructor with only curve sets offset to 0 and bounds to 0,0
    let anOffset = Adaptor2dOffsetCurve::with_curve(&myBaseLine);
    EXPECT_DOUBLE_EQ(anOffset.offset(), 0.0);
    EXPECT_DOUBLE_EQ(anOffset.first_parameter().unwrap(), 0.0);
    EXPECT_DOUBLE_EQ(anOffset.last_parameter().unwrap(), 0.0);
}

#[test]
fn Adaptor2d_OffsetCurveTest__ConstructWithOffset() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    EXPECT_DOUBLE_EQ(anOffset.offset(), 3.0);
    EXPECT_DOUBLE_EQ(anOffset.first_parameter().unwrap(), 0.0);
    EXPECT_DOUBLE_EQ(anOffset.last_parameter().unwrap(), 10.0);
}

#[test]
fn Adaptor2d_OffsetCurveTest__ConstructWithOffsetAndBounds() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_bounds(&myBaseLine, 2.0, 1.0, 8.0);
    EXPECT_DOUBLE_EQ(anOffset.offset(), 2.0);
    EXPECT_DOUBLE_EQ(anOffset.first_parameter().unwrap(), 1.0);
    EXPECT_DOUBLE_EQ(anOffset.last_parameter().unwrap(), 8.0);
}

#[test]
fn Adaptor2d_OffsetCurveTest__Value_PositiveOffset() {
    let myBaseLine = setup();
    // Offsetting a horizontal +X line by +3 shifts Y by -3
    // (OCCT 2D offset uses right-hand normal: Z cross tangent)
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    let aPnt = anOffset.value(5.0).unwrap();

    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), -3.0, precision::confusion());
}

#[test]
fn Adaptor2d_OffsetCurveTest__Value_NegativeOffset() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, -2.0).unwrap();
    let aPnt = anOffset.value(5.0).unwrap();

    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 2.0, precision::confusion());
}

#[test]
fn Adaptor2d_OffsetCurveTest__Value_ZeroOffset_MatchesBaseCurve() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 0.0).unwrap();

    let mut u: f64 = 0.0;
    while u <= 10.0 {
        let aOffPnt = anOffset.value(u).unwrap();
        let aBasePnt = myBaseLine.value(u).unwrap();
        assert!(aOffPnt.is_equal(aBasePnt, precision::confusion()));
        u += 2.0;
    }
}

#[test]
fn Adaptor2d_OffsetCurveTest__D1_OffsetLine() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();

    let (_aPnt, aVec) = anOffset.d1(5.0).unwrap();

    // First derivative of an offset line is same as the base line direction
    EXPECT_NEAR(aVec.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aVec.y(), 0.0, precision::confusion());
}

#[test]
fn Adaptor2d_OffsetCurveTest__Continuity() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    // Offset of CN line is CN
    assert_eq!(anOffset.continuity().unwrap(), Shape::CN);
}

#[test]
fn Adaptor2d_OffsetCurveTest__NbIntervals() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    assert_eq!(anOffset.nb_intervals(Shape::CN).unwrap(), 1);
}

#[test]
fn Adaptor2d_OffsetCurveTest__GetType_Line() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    assert_eq!(anOffset.get_type().unwrap(), CurveType::Line);
}

#[test]
fn Adaptor2d_OffsetCurveTest__IsNotClosed() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    assert!(!anOffset.is_closed().unwrap());
}

#[test]
fn Adaptor2d_OffsetCurveTest__IsNotPeriodic() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    assert!(!anOffset.is_periodic().unwrap());
}

#[test]
fn Adaptor2d_OffsetCurveTest__LoadCurve_ResetsOffset() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 5.0).unwrap();
    EXPECT_DOUBLE_EQ(anOffset.offset(), 5.0);

    // Load a new curve resets offset to 0
    let aNewLine: Handle<dyn Adaptor2dCurve2d> = Handle::new(Adaptor2dLine2d::new(
        Point2::new(0.0, 0.0),
        dir(0.0, 1.0),
        0.0,
        5.0,
    ))
    .cast();
    anOffset.load_curve(&aNewLine);
    EXPECT_DOUBLE_EQ(anOffset.offset(), 0.0);
}

#[test]
fn Adaptor2d_OffsetCurveTest__LoadOffset() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_curve(&myBaseLine);
    EXPECT_DOUBLE_EQ(anOffset.offset(), 0.0);

    anOffset.load_offset(7.5).unwrap();
    EXPECT_DOUBLE_EQ(anOffset.offset(), 7.5);
}

#[test]
fn Adaptor2d_OffsetCurveTest__LoadOffsetWithBounds() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_curve(&myBaseLine);
    anOffset.load_offset_bounded(4.0, 2.0, 6.0);

    EXPECT_DOUBLE_EQ(anOffset.offset(), 4.0);
    EXPECT_DOUBLE_EQ(anOffset.first_parameter().unwrap(), 2.0);
    EXPECT_DOUBLE_EQ(anOffset.last_parameter().unwrap(), 6.0);
}

#[test]
fn Adaptor2d_OffsetCurveTest__Curve_ReturnsBaseCurve() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    let aCurve = anOffset.curve();
    assert!(!aCurve.is_null());
    assert_eq!(aCurve.get_type().unwrap(), CurveType::Line);
}

#[test]
fn Adaptor2d_OffsetCurveTest__Trim() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    let aTrimmed = anOffset.trim(2.0, 7.0, precision::confusion()).unwrap();

    EXPECT_DOUBLE_EQ(aTrimmed.first_parameter().unwrap(), 2.0);
    EXPECT_DOUBLE_EQ(aTrimmed.last_parameter().unwrap(), 7.0);
}

#[test]
fn Adaptor2d_OffsetCurveTest__ShallowCopy() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_bounds(&myBaseLine, 3.0, 1.0, 9.0);
    let aCopy = anOffset.shallow_copy().unwrap();

    EXPECT_DOUBLE_EQ(aCopy.first_parameter().unwrap(), 1.0);
    EXPECT_DOUBLE_EQ(aCopy.last_parameter().unwrap(), 9.0);

    let aPntOrig = anOffset.value(5.0).unwrap();
    let aPntCopy = aCopy.value(5.0).unwrap();
    assert!(aPntOrig.is_equal(aPntCopy, precision::confusion()));
}

#[test]
fn Adaptor2d_OffsetCurveTest__Resolution() {
    let myBaseLine = setup();
    let anOffset = Adaptor2dOffsetCurve::with_offset(&myBaseLine, 3.0).unwrap();
    let aRes = anOffset.resolution(0.001).unwrap();
    assert!(aRes > 0.0);
}
