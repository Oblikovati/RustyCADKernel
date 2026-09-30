//! Direct translation of GeomBndLib_SurfaceOfRevolution_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{array1, curve, dir, get3, pnt, surface};
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomParabola, GeomSurface,
    GeomSurfaceOfRevolution,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::bnd_lib::AddSurface;
use rustycad_tkgeombase::geom_bnd_lib::Surface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Axis1, Axis2, Direction3, Point3};
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

// `new Geom_SurfaceOfRevolution(theBasis, gp_Ax1(gp::Origin(), theAxisDir))`.
fn revolution(theBasis: Handle<dyn GeomCurve>, theAxisDir: Direction3) -> Handle<dyn GeomSurface> {
    let anAxis = Axis1::new(gp::origin(), theAxisDir);
    surface(GeomSurfaceOfRevolution::new(&theBasis, anAxis).unwrap())
}

fn line(theOrigin: Point3, theDir: Direction3) -> Handle<dyn GeomCurve> {
    curve(GeomLine::from_point(theOrigin, theDir))
}

// `gp_Ax2(gp_Pnt(5.0, 0.0, 0.0), gp::DY(), gp::DX())`: the XZ frame of the circle and
// ellipse tests.
fn xz_frame() -> Axis2 {
    Axis2::with_x_direction(pnt(5.0, 0.0, 0.0), gp::dy(), gp::dx()).unwrap()
}

fn bspline() -> Handle<dyn GeomCurve> {
    let aPoles = array1(&[
        pnt(3.0, 0.0, 0.0),
        pnt(5.0, 0.0, 3.0),
        pnt(4.0, 0.0, 7.0),
        pnt(6.0, 0.0, 10.0),
    ]);
    let aKnots = array1(&[0.0, 0.5, 1.0]);
    let aMults = array1(&[3, 1, 3]);
    curve(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap())
}

// GeomBndLib_Surface(aRevSurf).Add against BndLib_AddSurface::Add on a
// GeomAdaptor_Surface over the patch.
fn boxes(theSurf: &Handle<dyn GeomSurface>, thePatch: [f64; 4]) -> (BoundingBox3, BoundingBox3) {
    let anAdaptor = GeomAdaptorSurface::with_surface(theSurf).unwrap();
    let aTol = precision::confusion();
    let mut aNewBox = BoundingBox3::default();
    Surface::from_surface(theSurf)
        .unwrap()
        .add_range(thePatch, aTol, &mut aNewBox)
        .unwrap();
    let mut anOldBox = BoundingBox3::default();
    AddSurface::add_range(&anAdaptor, thePatch, aTol, &mut anOldBox).unwrap();
    (aNewBox, anOldBox)
}

// The body shared by the comparison tests: no larger than BndLib (within 0.5) and
// containing the `theNbU x theNbV` samples of the patch.
fn check(theSurf: &Handle<dyn GeomSurface>, thePatch: [f64; 4], theNbU: i32, theNbV: i32) {
    let (aNewBox, anOldBox) = boxes(theSurf, thePatch);
    ExpectNoLarger(&aNewBox, &anOldBox, 0.5);
    ExpectContainsSurface(&aNewBox, theSurf, thePatch, theNbU, theNbV);
}

// =========================================================================
// Line around Z axis (creates cylinder-like surface)
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__LineAroundZ_Full_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(5.0, 0.0, 0.0), gp::dz()), gp::dz());
    check(&aRevSurf, [0.0, 2.0 * PI, 0.0, 10.0], 36, 10);
}

#[test]
fn GeomBndLib_RevolutionTest__LineAroundZ_HalfTurn_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(5.0, 0.0, 0.0), gp::dz()), gp::dz());
    check(&aRevSurf, [0.0, PI, 0.0, 5.0], 18, 5);
}

#[test]
fn GeomBndLib_RevolutionTest__InfiniteV_OpenInAxisDirection() {
    // Line along Z through (5, 0, 0) revolved around Z axis (full 2*pi).
    // P(U, V) = (5*cos(U), 5*sin(U), V); V in (-inf, +inf).
    // Expected: X in [-5, 5], Y in [-5, 5], Z open both ways.
    let aRevSurf = revolution(line(pnt(5.0, 0.0, 0.0), gp::dz()), gp::dz());

    let mut aNewBox = BoundingBox3::default();
    Surface::from_surface(&aRevSurf)
        .unwrap()
        .add_range(
            [0.0, 2.0 * PI, -precision::infinite(), precision::infinite()],
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
    EXPECT_NEAR(aXmin, -5.0, aTol);
    EXPECT_NEAR(aXmax, 5.0, aTol);
    EXPECT_NEAR(aYmin, -5.0, aTol);
    EXPECT_NEAR(aYmax, 5.0, aTol);
}

#[test]
fn GeomBndLib_RevolutionTest__LineAroundZ_QuarterTurn_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(5.0, 0.0, 0.0), gp::dz()), gp::dz());
    check(&aRevSurf, [0.0, PI / 2.0, 0.0, 5.0], 9, 5);
}

// =========================================================================
// Line around X axis
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__LineAroundX_Full_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(0.0, 3.0, 0.0), gp::dx()), gp::dx());
    check(&aRevSurf, [0.0, 2.0 * PI, 0.0, 8.0], 36, 10);
}

// =========================================================================
// Line around tilted axis
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__LineAroundTiltedAxis_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(4.0, 0.0, 0.0), gp::dz()), dir(1.0, 1.0, 0.0));
    check(&aRevSurf, [0.0, 2.0 * PI, 0.0, 5.0], 36, 10);
}

// =========================================================================
// Circle around Z axis (creates torus-like surface)
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__CircleAroundZ_Full_CompareWithBndLib() {
    // Circle in XZ plane, center at (5, 0, 0), radius 2
    let aCircle = curve(GeomCircle::new(xz_frame(), 2.0).unwrap());
    let aRevSurf = revolution(aCircle, gp::dz());
    check(&aRevSurf, [0.0, 2.0 * PI, 0.0, 2.0 * PI], 36, 36);
}

#[test]
fn GeomBndLib_RevolutionTest__CircleAroundZ_HalfTurn_CompareWithBndLib() {
    let aCircle = curve(GeomCircle::new(xz_frame(), 2.0).unwrap());
    let aRevSurf = revolution(aCircle, gp::dz());
    check(&aRevSurf, [0.0, PI, 0.0, 2.0 * PI], 18, 36);
}

// =========================================================================
// Parabola around Z axis
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__ParabolaAroundZ_CompareWithBndLib() {
    let aParab = curve(GeomParabola::new(gp::xoy(), 2.0).unwrap());
    let aRevSurf = revolution(aParab, gp::dz());
    check(&aRevSurf, [0.0, 2.0 * PI, -3.0, 3.0], 36, 10);
}

// =========================================================================
// Ellipse around Z axis
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__EllipseAroundZ_CompareWithBndLib() {
    // Ellipse in XZ plane
    let anEllipse = curve(GeomEllipse::new(xz_frame(), 3.0, 1.5).unwrap());
    let aRevSurf = revolution(anEllipse, gp::dz());
    check(&aRevSurf, [0.0, 2.0 * PI, 0.0, 2.0 * PI], 36, 36);
}

// =========================================================================
// BSpline curve around Z axis
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__BSplineAroundZ_CompareWithBndLib() {
    let aRevSurf = revolution(bspline(), gp::dz());
    check(&aRevSurf, [0.0, 2.0 * PI, 0.0, 1.0], 36, 10);
}

#[test]
fn GeomBndLib_RevolutionTest__BSplineAroundZ_HalfTurn_CompareWithBndLib() {
    let aRevSurf = revolution(bspline(), gp::dz());
    check(&aRevSurf, [0.0, PI, 0.0, 1.0], 18, 10);
}

// =========================================================================
// Offset line from axis (different radius)
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__LineSmallRadius_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(0.5, 0.0, 0.0), gp::dz()), gp::dz());
    check(&aRevSurf, [0.0, 2.0 * PI, -5.0, 5.0], 36, 10);
}

#[test]
fn GeomBndLib_RevolutionTest__LineLargeRadius_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(100.0, 0.0, 0.0), gp::dz()), gp::dz());
    check(&aRevSurf, [0.0, 2.0 * PI, 0.0, 10.0], 36, 10);
}

// =========================================================================
// V-range partial
// =========================================================================

#[test]
fn GeomBndLib_RevolutionTest__LineAroundZ_PartialV_CompareWithBndLib() {
    let aRevSurf = revolution(line(pnt(5.0, 0.0, 0.0), gp::dz()), gp::dz());
    // Only part of V range
    check(&aRevSurf, [0.0, 2.0 * PI, 3.0, 7.0], 36, 10);
}
