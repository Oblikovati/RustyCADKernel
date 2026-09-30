//! Direct translation of GeomBndLib_OffsetCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod bnd_support;
use bnd_support::{array1, curve, dir, get3, pnt};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomOffsetCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkgeombase::geom_bnd_lib::Curve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Axis2, Direction3};
use std::f64::consts::PI;

// Helper: verify new box is no larger than old box (tighter or equal).
fn ExpectNoLarger(theNew: &BoundingBox3, theOld: &BoundingBox3, theTol: f64) {
    let [aXminN, aXmaxN, aYminN, aYmaxN, aZminN, aZmaxN] = get3(theNew);
    let [aXminO, aXmaxO, aYminO, aYmaxO, aZminO, aZmaxO] = get3(theOld);
    assert!(aXminN >= aXminO - theTol, "New Xmin is smaller than old");
    assert!(aXmaxN <= aXmaxO + theTol, "New Xmax is larger than old");
    assert!(aYminN >= aYminO - theTol, "New Ymin is smaller than old");
    assert!(aYmaxN <= aYmaxO + theTol, "New Ymax is larger than old");
    assert!(aZminN >= aZminO - theTol, "New Zmin is smaller than old");
    assert!(aZmaxN <= aZmaxO + theTol, "New Zmax is larger than old");
}

// Helper: verify box contains all sampled points of the offset curve.
fn ExpectContainsCurve(
    theBox: &BoundingBox3,
    theCurve: &Handle<dyn GeomCurve>,
    theU1: f64,
    theU2: f64,
    theNbSamples: i32,
) {
    let aStep = (theU2 - theU1) / f64::from(theNbSamples);
    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = theBox.get().unwrap();
    for i in 0..=theNbSamples {
        let aU = theU1 + f64::from(i) * aStep;
        let aPnt = theCurve.get().unwrap().value(aU).unwrap();
        let aTol = precision::confusion();
        assert!(aPnt.x() >= aXmin - aTol, "Point outside box at U={aU}");
        assert!(aPnt.x() <= aXmax + aTol, "Point outside box at U={aU}");
        assert!(aPnt.y() >= aYmin - aTol, "Point outside box at U={aU}");
        assert!(aPnt.y() <= aYmax + aTol, "Point outside box at U={aU}");
        assert!(aPnt.z() >= aZmin - aTol, "Point outside box at U={aU}");
        assert!(aPnt.z() <= aZmax + aTol, "Point outside box at U={aU}");
    }
}

fn offset(
    theBasis: &Handle<dyn GeomCurve>,
    theOffset: f64,
    theDir: Direction3,
) -> Handle<dyn GeomCurve> {
    curve(GeomOffsetCurve::new(theBasis, theOffset, theDir, false).unwrap())
}

fn circle(theRadius: f64) -> Handle<dyn GeomCurve> {
    curve(GeomCircle::new(gp::xoy(), theRadius).unwrap())
}

// GeomBndLib_Curve(anOff) against BndLib_Add3dCurve on a GeomAdaptor_Curve: `Add`
// over the natural range, `AddOptimal` when `theOptimal`, or `Add(U1, U2)` for a range.
fn boxes(
    theCurve: &Handle<dyn GeomCurve>,
    theRange: Option<(f64, f64)>,
    theOptimal: bool,
) -> (BoundingBox3, BoundingBox3) {
    let anAdaptor = GeomAdaptorCurve::with_curve(theCurve).unwrap();
    let anEval = Curve::from_curve(theCurve).unwrap();
    let aTol = precision::confusion();
    let (mut aNewBox, mut anOldBox) = (BoundingBox3::default(), BoundingBox3::default());
    match (theRange, theOptimal) {
        (Some((aU1, aU2)), _) => {
            anEval.add_range(aU1, aU2, aTol, &mut aNewBox).unwrap();
            Add3dCurve::add_range(&anAdaptor, aU1, aU2, aTol, &mut anOldBox).unwrap();
        }
        (None, true) => {
            anEval.add_optimal(aTol, &mut aNewBox).unwrap();
            Add3dCurve::add_optimal(&anAdaptor, aTol, &mut anOldBox).unwrap();
        }
        (None, false) => {
            anEval.add(aTol, &mut aNewBox).unwrap();
            Add3dCurve::add(&anAdaptor, aTol, &mut anOldBox).unwrap();
        }
    }
    (aNewBox, anOldBox)
}

// =========================================================================
// Circle offset in Z direction (displacement in XY plane)
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__CircleOffsetZ_Full_CompareWithBndLib() {
    let anOff = offset(&circle(5.0), 2.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, None, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

#[test]
fn GeomBndLib_OffsetCurveTest__CircleOffsetZ_Arc_CompareWithBndLib() {
    let anOff = offset(&circle(5.0), 3.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, Some((0.0, PI / 2.0)), false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, PI / 2.0, 50);
}

// =========================================================================
// Circle offset in X direction (displacement in YZ plane)
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__CircleOffsetX_Full_CompareWithBndLib() {
    let anOff = offset(&circle(7.0), 2.0, gp::dx());
    let (aNewBox, anOldBox) = boxes(&anOff, None, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

#[test]
fn GeomBndLib_OffsetCurveTest__CircleOffsetX_Arc_CompareWithBndLib() {
    let anOff = offset(&circle(7.0), 4.0, gp::dx());
    let (aNewBox, anOldBox) = boxes(&anOff, Some((PI / 4.0, 3.0 * PI / 4.0)), false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, PI / 4.0, 3.0 * PI / 4.0, 50);
}

// =========================================================================
// Circle offset in Y direction - degenerate case
// Offset direction Y is in the circle plane (XOY), so T(t) x V can be zero.
// Old BndLib throws "GeomAdaptor_Curve::EvalD0: evaluation failed".
// Test only containment for our implementation.
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__CircleOffsetY_Full_Containment() {
    let anOff = offset(&circle(3.0), 1.5, gp::dy());

    let mut aNewBox = BoundingBox3::default();
    Curve::from_curve(&anOff)
        .unwrap()
        .add(precision::confusion(), &mut aNewBox)
        .unwrap();

    assert!(!aNewBox.is_void());
}

// =========================================================================
// Circle offset in diagonal direction
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__CircleOffsetDiagonal_CompareWithBndLib() {
    let anOff = offset(&circle(5.0), 2.0, dir(1.0, 1.0, 1.0));
    let (aNewBox, anOldBox) = boxes(&anOff, None, true);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

// =========================================================================
// Negative offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__CircleNegativeOffset_CompareWithBndLib() {
    let anOff = offset(&circle(10.0), -3.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, None, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

// =========================================================================
// Ellipse offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__EllipseOffsetZ_Full_CompareWithBndLib() {
    let anEllipse = curve(GeomEllipse::new(gp::xoy(), 8.0, 3.0).unwrap());
    let anOff = offset(&anEllipse, 2.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, None, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

#[test]
fn GeomBndLib_OffsetCurveTest__EllipseOffsetX_Arc_CompareWithBndLib() {
    let anEllipse = curve(GeomEllipse::new(gp::xoy(), 8.0, 3.0).unwrap());
    let anOff = offset(&anEllipse, 1.5, gp::dx());
    let (aNewBox, anOldBox) = boxes(&anOff, Some((0.0, PI)), false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, PI, 50);
}

// =========================================================================
// Line offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__LineOffset_CompareWithBndLib() {
    let aLine = curve(GeomLine::from_point(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)));
    let anOff = offset(&aLine, 3.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, Some((0.0, 10.0)), false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 10.0, 50);
}

// =========================================================================
// BSpline offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__BSplineOffset_CompareWithBndLib() {
    let aPoles = array1(&[
        pnt(0.0, 0.0, 0.0),
        pnt(3.0, 5.0, 1.0),
        pnt(7.0, -2.0, 2.0),
        pnt(10.0, 0.0, 0.0),
    ]);
    let aKnots = array1(&[0.0, 0.5, 1.0]);
    let aMults = array1(&[3, 1, 3]);

    let aBSpl = curve(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap());
    let anOff = offset(&aBSpl, 1.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, None, true);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 1.0, 50);
}

// =========================================================================
// Large offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__LargeOffset_CompareWithBndLib() {
    let anOff = offset(&circle(2.0), 20.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, None, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

// =========================================================================
// Non-planar circle with offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetCurveTest__TiltedCircle_CompareWithBndLib() {
    // Circle in a tilted plane
    let anAx2 = Axis2::new(pnt(1.0, 2.0, 3.0), dir(1.0, 1.0, 1.0)).unwrap();
    let aCircle = curve(GeomCircle::new(anAx2, 4.0).unwrap());
    let anOff = offset(&aCircle, 1.5, dir(1.0, 1.0, 1.0));
    let (aNewBox, anOldBox) = boxes(&anOff, None, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsCurve(&aNewBox, &anOff, 0.0, 2.0 * PI, 100);
}

#[test]
fn GeomBndLib_OffsetCurveTest__InfiniteRange_OpennessPreserved() {
    let aLine = curve(GeomLine::from_point(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)));
    let anOff = offset(&aLine, 3.0, gp::dz());
    let (aNewBox, anOldBox) = boxes(&anOff, Some((-precision::infinite(), 10.0)), false);

    assert!(!aNewBox.is_void());
    assert!(aNewBox.is_open_xmin());
    assert!(!aNewBox.is_open_xmax());
    assert!(!aNewBox.is_open_ymin());
    assert!(!aNewBox.is_open_ymax());
    assert!(!aNewBox.is_open_zmin());
    assert!(!aNewBox.is_open_zmax());
    // BndLib now delegates to GeomBndLib, so it also correctly computes the open X min.
    assert!(anOldBox.is_open_xmin());
}
