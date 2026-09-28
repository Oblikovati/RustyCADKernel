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

//! Direct translation of GeomBndLib_OffsetCurve2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod bnd_support;
use bnd_support::{array1, curve2d, dir2d, get2, pnt2d};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dLine, Geom2dOffsetCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkgeombase::geom_bnd_lib::Curve2d;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Axis2d, Axis22d, Point2};
use std::f64::consts::PI;

// Helper: verify new 2D box is no larger than old box (tighter or equal).
fn ExpectNoLarger2d(theNew: &BoundingBox2, theOld: &BoundingBox2, theTol: f64) {
    let [aXminN, aXmaxN, aYminN, aYmaxN] = get2(theNew);
    let [aXminO, aXmaxO, aYminO, aYmaxO] = get2(theOld);
    assert!(aXminN >= aXminO - theTol, "New Xmin is smaller than old");
    assert!(aXmaxN <= aXmaxO + theTol, "New Xmax is larger than old");
    assert!(aYminN >= aYminO - theTol, "New Ymin is smaller than old");
    assert!(aYmaxN <= aYmaxO + theTol, "New Ymax is larger than old");
}

// Helper: verify 2D box contains all sampled points of the offset curve.
fn ExpectContainsCurve2d(
    theBox: &BoundingBox2,
    theCurve: &Handle<dyn Geom2dCurve>,
    theU1: f64,
    theU2: f64,
    theNbSamples: i32,
) {
    let aStep = (theU2 - theU1) / f64::from(theNbSamples);
    let [aXmin, aYmin, aXmax, aYmax] = theBox.get().unwrap();
    for i in 0..=theNbSamples {
        let aU = theU1 + f64::from(i) * aStep;
        let aPnt: Point2 = theCurve.get().unwrap().value(aU).unwrap();
        let aTol = precision::confusion();
        assert!(aPnt.x() >= aXmin - aTol, "Point outside box at U={aU}");
        assert!(aPnt.x() <= aXmax + aTol, "Point outside box at U={aU}");
        assert!(aPnt.y() >= aYmin - aTol, "Point outside box at U={aU}");
        assert!(aPnt.y() <= aYmax + aTol, "Point outside box at U={aU}");
    }
}

fn offset(theBasis: &Handle<dyn Geom2dCurve>, theOffset: f64) -> Handle<dyn Geom2dCurve> {
    curve2d(Geom2dOffsetCurve::new(theBasis, theOffset, false).unwrap())
}

fn circle(theCentre: Point2, theRadius: f64) -> Handle<dyn Geom2dCurve> {
    curve2d(
        Geom2dCircle::from_axis(Axis2d::new(theCentre, dir2d(1.0, 0.0)), theRadius, true).unwrap(),
    )
}

fn ellipse() -> Handle<dyn Geom2dCurve> {
    let aFrame =
        Axis22d::with_directions(pnt2d(0.0, 0.0), dir2d(1.0, 0.0), dir2d(0.0, 1.0)).unwrap();
    curve2d(Geom2dEllipse::new(aFrame, 8.0, 3.0).unwrap())
}

// GeomBndLib_Curve2d(anOff).Add against BndLib_Add2dCurve::Add on a Geom2dAdaptor_Curve,
// over the natural range or `theRange`.
fn boxes(
    theCurve: &Handle<dyn Geom2dCurve>,
    theRange: Option<(f64, f64)>,
) -> (BoundingBox2, BoundingBox2) {
    let anAdaptor = Geom2dAdaptorCurve::with_curve(theCurve).unwrap();
    let anEval = Curve2d::from_curve(theCurve).unwrap();
    let aTol = precision::confusion();
    let (mut aNewBox, mut anOldBox) = (BoundingBox2::default(), BoundingBox2::default());
    if let Some((aU1, aU2)) = theRange {
        anEval.add_range(aU1, aU2, aTol, &mut aNewBox).unwrap();
        Add2dCurve::add_adaptor_range(&anAdaptor, aU1, aU2, aTol, &mut anOldBox).unwrap();
    } else {
        anEval.add(aTol, &mut aNewBox).unwrap();
        Add2dCurve::add_adaptor(&anAdaptor, aTol, &mut anOldBox).unwrap();
    }
    (aNewBox, anOldBox)
}

// =========================================================================
// Circle offset (positive)
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurve2dTest__Circle_PositiveOffset_Full_CompareWithBndLib() {
    let anOff = offset(&circle(pnt2d(0.0, 0.0), 5.0), 2.0);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

#[test]
fn GeomBndLib_OffsetCurve2dTest__Circle_PositiveOffset_Arc_CompareWithBndLib() {
    let anOff = offset(&circle(pnt2d(0.0, 0.0), 5.0), 2.0);
    let (aNewBox, anOldBox) = boxes(&anOff, Some((0.0, PI / 2.0)));
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, PI / 2.0, 50);
}

// =========================================================================
// Circle offset (negative)
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurve2dTest__Circle_NegativeOffset_Full_CompareWithBndLib() {
    let anOff = offset(&circle(pnt2d(0.0, 0.0), 10.0), -3.0);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

#[test]
fn GeomBndLib_OffsetCurve2dTest__Circle_NegativeOffset_Arc_CompareWithBndLib() {
    let anOff = offset(&circle(pnt2d(0.0, 0.0), 10.0), -3.0);
    let (aNewBox, anOldBox) = boxes(&anOff, Some((PI / 4.0, PI)));
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, PI / 4.0, PI, 50);
}

// =========================================================================
// Ellipse offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurve2dTest__Ellipse_Full_CompareWithBndLib() {
    let anOff = offset(&ellipse(), 1.5);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

#[test]
fn GeomBndLib_OffsetCurve2dTest__Ellipse_Arc_OptimalCompareWithBndLib() {
    let anOff = offset(&ellipse(), 1.5);

    let mut aNewBox = BoundingBox2::default();
    Curve2d::from_curve(&anOff)
        .unwrap()
        .add_optimal_range(0.0, PI, precision::confusion(), &mut aNewBox)
        .unwrap();

    let mut anOldBox = BoundingBox2::default();
    Add2dCurve::add_optimal(&anOff, 0.0, PI, precision::confusion(), &mut anOldBox).unwrap();

    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, PI, 50);
}

// =========================================================================
// Line offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurve2dTest__Line_CompareWithBndLib() {
    let aLine = curve2d(Geom2dLine::from_point(pnt2d(0.0, 0.0), dir2d(1.0, 0.0)));
    let anOff = offset(&aLine, 3.0);
    let (aNewBox, anOldBox) = boxes(&anOff, Some((0.0, 10.0)));
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, 10.0, 50);
}

// =========================================================================
// BSpline offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurve2dTest__BSpline_OptimalCompareWithBndLib() {
    let aPoles = array1(&[
        pnt2d(0.0, 0.0),
        pnt2d(3.0, 5.0),
        pnt2d(7.0, -2.0),
        pnt2d(10.0, 0.0),
    ]);
    let aKnots = array1(&[0.0, 0.5, 1.0]);
    let aMults = array1(&[3, 1, 3]);

    let aBSpl = curve2d(Geom2dBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap());
    let anOff = offset(&aBSpl, 1.0);

    let mut aNewBox = BoundingBox2::default();
    Curve2d::from_curve(&anOff)
        .unwrap()
        .add_optimal(precision::confusion(), &mut aNewBox)
        .unwrap();

    let mut anOldBox = BoundingBox2::default();
    Add2dCurve::add_optimal(&anOff, 0.0, 1.0, precision::confusion(), &mut anOldBox).unwrap();

    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, 1.0, 50);
}

// =========================================================================
// Large offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurve2dTest__LargeOffset_CompareWithBndLib() {
    let anOff = offset(&circle(pnt2d(0.0, 0.0), 2.0), 20.0);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

// =========================================================================
// Off-center circle with offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurve2dTest__OffCenterCircle_CompareWithBndLib() {
    let anOff = offset(&circle(pnt2d(5.0, 3.0), 4.0), 1.0);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger2d(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve2d(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}
