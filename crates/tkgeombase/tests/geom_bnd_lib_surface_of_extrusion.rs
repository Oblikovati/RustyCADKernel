//! Direct translation of GeomBndLib_SurfaceOfExtrusion_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{array1, curve, dir, get3, pnt, surface};
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomSurface,
    GeomSurfaceOfLinearExtrusion,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::bnd_lib::AddSurface;
use rustycad_tkgeombase::geom_bnd_lib::Surface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Direction3};
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

// Helper: verify box contains all sampled points of the surface.
fn ExpectContainsSurface(
    theBox: &BoundingBox3,
    theSurf: &Handle<dyn GeomSurface>,
    [theUMin, theUMax, theVMin, theVMax]: [f64; 4],
    theNbU: i32,
    theNbV: i32,
) {
    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = theBox.get().unwrap();
    let aUStep = (theUMax - theUMin) / f64::from(theNbU);
    let aVStep = (theVMax - theVMin) / f64::from(theNbV);
    let aTol = precision::confusion();
    for i in 0..=theNbU {
        let aU = theUMin + f64::from(i) * aUStep;
        for j in 0..=theNbV {
            let aV = theVMin + f64::from(j) * aVStep;
            let aPnt = theSurf.get().unwrap().value(aU, aV).unwrap();
            assert!(
                aPnt.x() >= aXmin - aTol,
                "Point outside box at U={aU} V={aV}"
            );
            assert!(
                aPnt.x() <= aXmax + aTol,
                "Point outside box at U={aU} V={aV}"
            );
            assert!(
                aPnt.y() >= aYmin - aTol,
                "Point outside box at U={aU} V={aV}"
            );
            assert!(
                aPnt.y() <= aYmax + aTol,
                "Point outside box at U={aU} V={aV}"
            );
            assert!(
                aPnt.z() >= aZmin - aTol,
                "Point outside box at U={aU} V={aV}"
            );
            assert!(
                aPnt.z() <= aZmax + aTol,
                "Point outside box at U={aU} V={aV}"
            );
        }
    }
}

fn extrusion(theBasis: Handle<dyn GeomCurve>, theDir: Direction3) -> Handle<dyn GeomSurface> {
    surface(GeomSurfaceOfLinearExtrusion::new(&theBasis, theDir).unwrap())
}

fn circle() -> Handle<dyn GeomCurve> {
    curve(GeomCircle::new(gp::xoy(), 5.0).unwrap())
}

fn ellipse() -> Handle<dyn GeomCurve> {
    curve(GeomEllipse::new(gp::xoy(), 8.0, 3.0).unwrap())
}

fn bspline() -> Handle<dyn GeomCurve> {
    let aPoles = array1(&[
        pnt(0.0, 0.0, 0.0),
        pnt(3.0, 5.0, 0.0),
        pnt(7.0, -2.0, 0.0),
        pnt(10.0, 3.0, 0.0),
    ]);
    let aKnots = array1(&[0.0, 0.5, 1.0]);
    let aMults = array1(&[3, 1, 3]);
    curve(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap())
}

// GeomBndLib_Surface(anExtr) and BndLib_AddSurface on a GeomAdaptor_Surface over the
// patch: `Add`, or `AddOptimal` when `theOptimal`.
fn boxes(
    theSurf: &Handle<dyn GeomSurface>,
    thePatch: [f64; 4],
    theOptimal: bool,
) -> (BoundingBox3, BoundingBox3) {
    let anAdaptor = GeomAdaptorSurface::with_surface(theSurf).unwrap();
    let anEval = Surface::from_surface(theSurf).unwrap();
    let aTol = precision::confusion();
    let (mut aNewBox, mut anOldBox) = (BoundingBox3::default(), BoundingBox3::default());
    if theOptimal {
        anEval
            .add_optimal_range(thePatch, aTol, &mut aNewBox)
            .unwrap();
        AddSurface::add_optimal_range(&anAdaptor, thePatch, aTol, &mut anOldBox).unwrap();
    } else {
        anEval.add_range(thePatch, aTol, &mut aNewBox).unwrap();
        AddSurface::add_range(&anAdaptor, thePatch, aTol, &mut anOldBox).unwrap();
    }
    (aNewBox, anOldBox)
}

// =========================================================================
// Circle along Z
// =========================================================================

#[test]
fn GeomBndLib_ExtrusionTest__CircleAlongZ_Full_CompareWithBndLib() {
    let anExtr = extrusion(circle(), gp::dz());
    let aPatch = [0.0, 2.0 * PI, 0.0, 10.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 36, 10);
}

#[test]
fn GeomBndLib_ExtrusionTest__CircleAlongZ_Arc_CompareWithBndLib() {
    let anExtr = extrusion(circle(), gp::dz());
    let aPatch = [0.0, PI, 0.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 18, 5);
}

#[test]
fn GeomBndLib_ExtrusionTest__InfiniteU_OpenInExtrusionBasisDirection() {
    // Line along Z through (5, 0, 0) extruded along X.
    // P(U, V) = (5 + V, 0, U); U in (-inf, +inf), V in [0, 10].
    // Expected: X in [5, 15], Y finite (= 0), Z open both ways.
    let aLine = curve(GeomLine::from_point(pnt(5.0, 0.0, 0.0), gp::dz()));
    let anExtr = extrusion(aLine, gp::dx());

    let mut aNewBox = BoundingBox3::default();
    Surface::from_surface(&anExtr)
        .unwrap()
        .add_range(
            [-precision::infinite(), precision::infinite(), 0.0, 10.0],
            precision::confusion(),
            &mut aNewBox,
        )
        .unwrap();

    assert!(!aNewBox.is_void());
    assert!(!aNewBox.is_whole());
    assert!(!aNewBox.is_open_xmin());
    assert!(!aNewBox.is_open_xmax());
    assert!(!aNewBox.is_open_ymin());
    assert!(!aNewBox.is_open_ymax());
    assert!(aNewBox.is_open_zmin());
    assert!(aNewBox.is_open_zmax());
    let [aXmin, aXmax, aYmin, aYmax, _aZmin, _aZmax] = get3(&aNewBox);
    let aTol = 2.0 * precision::confusion(); // box is enlarged by theTol on construction
    EXPECT_NEAR(aXmin, 5.0, aTol);
    EXPECT_NEAR(aXmax, 15.0, aTol);
    EXPECT_NEAR(aYmin, 0.0, aTol);
    EXPECT_NEAR(aYmax, 0.0, aTol);
}

// =========================================================================
// Circle along diagonal
// =========================================================================

#[test]
fn GeomBndLib_ExtrusionTest__CircleAlongDiagonal_CompareWithBndLib() {
    let anExtr = extrusion(circle(), dir(1.0, 0.0, 1.0));
    let aPatch = [0.0, 2.0 * PI, 0.0, 10.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 36, 10);
}

// =========================================================================
// Ellipse along Z
// =========================================================================

#[test]
fn GeomBndLib_ExtrusionTest__EllipseAlongZ_CompareWithBndLib() {
    let anExtr = extrusion(ellipse(), gp::dz());
    let aPatch = [0.0, 2.0 * PI, 0.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 36, 5);
}

#[test]
fn GeomBndLib_ExtrusionTest__EllipseAlongZ_Arc_CompareWithBndLib() {
    let anExtr = extrusion(ellipse(), gp::dz());
    let aPatch = [0.0, PI, -2.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 18, 7);
}

// =========================================================================
// Line along Z (planar surface)
// =========================================================================

#[test]
fn GeomBndLib_ExtrusionTest__LineAlongZ_CompareWithBndLib() {
    let aLine = curve(GeomLine::from_point(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)));
    let anExtr = extrusion(aLine, gp::dz());
    let aPatch = [0.0, 10.0, 0.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 10, 5);
}

// =========================================================================
// Line along diagonal
// =========================================================================

#[test]
fn GeomBndLib_ExtrusionTest__LineAlongDiagonal_CompareWithBndLib() {
    let aLine = curve(GeomLine::from_point(pnt(0.0, 0.0, 0.0), dir(0.0, 1.0, 0.0)));
    let anExtr = extrusion(aLine, dir(1.0, 0.0, 1.0));
    let aPatch = [0.0, 5.0, 0.0, 10.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 5, 10);
}

// =========================================================================
// BSpline along Z
// =========================================================================

#[test]
fn GeomBndLib_ExtrusionTest__BSplineAlongZ_CompareWithBndLib() {
    let anExtr = extrusion(bspline(), gp::dz());
    let aPatch = [0.0, 1.0, 0.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, true);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 10, 5);
}

#[test]
fn GeomBndLib_ExtrusionTest__BSplineAlongDiagonal_CompareWithBndLib() {
    let anExtr = extrusion(bspline(), dir(1.0, 1.0, 1.0));
    let aPatch = [0.0, 1.0, -3.0, 7.0];
    let mut aNewBox = BoundingBox3::default();
    Surface::from_surface(&anExtr)
        .unwrap()
        .add_optimal_range(aPatch, precision::confusion(), &mut aNewBox)
        .unwrap();

    // P(u,v) = C(u) + v*(1,1,1)/sqrt(3), u in [0,1], v in [-3, 7].
    // BSpline C(0)=(0,0,0), C(1)=(10,3,0). Extrusion shifts: v_min/sqrt(3) = -sqrt(3),
    // v_max/sqrt(3) = 7/sqrt(3). Reference bounds derive from endpoint extremes + extrusion range.
    let aSqrt3 = 3.0_f64.sqrt();
    let aVminShift = -3.0 / aSqrt3; // ~= -1.7321
    let aVmaxShift = 7.0 / aSqrt3; // ~=  4.0415
    let aTolRef = 1e-3;
    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aNewBox);
    EXPECT_NEAR(aXmin, 0.0 + aVminShift, aTolRef);
    EXPECT_NEAR(aXmax, 10.0 + aVmaxShift, aTolRef);
    EXPECT_NEAR(aYmin, 0.0 + aVminShift, aTolRef);
    EXPECT_NEAR(aYmax, 3.0 + aVmaxShift, aTolRef);
    EXPECT_NEAR(aZmin, aVminShift, aTolRef);
    EXPECT_NEAR(aZmax, aVmaxShift, aTolRef);
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 10, 10);
}

// =========================================================================
// Negative V range
// =========================================================================

#[test]
fn GeomBndLib_ExtrusionTest__CircleNegativeV_CompareWithBndLib() {
    let anExtr = extrusion(circle(), gp::dz());
    let aPatch = [0.0, 2.0 * PI, -5.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anExtr, aPatch, false);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsSurface(&aNewBox, &anExtr, aPatch, 36, 10);
}
