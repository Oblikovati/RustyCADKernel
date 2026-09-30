//! Direct translation of GeomBndLib_OffsetSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{get3, surface};
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomCylindricalSurface, GeomOffsetSurface, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::bnd_lib::AddSurface;
use rustycad_tkgeombase::geom_bnd_lib::Surface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Axis3};
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

// Helper: verify box contains all sampled points of the offset surface.
fn ExpectContainsOffsetSurface(
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

fn xoy() -> Axis3 {
    gp::xoy().axis3()
}

fn offset(theBasis: Handle<dyn GeomSurface>, theOffset: f64) -> Handle<dyn GeomSurface> {
    surface(GeomOffsetSurface::new(&theBasis, theOffset, false).unwrap())
}

fn sphere(theRadius: f64) -> Handle<dyn GeomSurface> {
    surface(GeomSphericalSurface::new(xoy(), theRadius).unwrap())
}

fn cylinder(theRadius: f64) -> Handle<dyn GeomSurface> {
    surface(GeomCylindricalSurface::new(xoy(), theRadius).unwrap())
}

fn torus() -> Handle<dyn GeomSurface> {
    surface(GeomToroidalSurface::new(xoy(), 10.0, 3.0).unwrap())
}

fn plane() -> Handle<dyn GeomSurface> {
    surface(GeomPlane::new(xoy()))
}

// GeomBndLib_Surface(anOff) and BndLib_AddSurface on a GeomAdaptor_Surface: `Add` over
// the patch, or the natural bounds when None.
fn boxes(
    theSurf: &Handle<dyn GeomSurface>,
    thePatch: Option<[f64; 4]>,
) -> (BoundingBox3, BoundingBox3) {
    let anAdaptor = GeomAdaptorSurface::with_surface(theSurf).unwrap();
    let anEval = Surface::from_surface(theSurf).unwrap();
    let aTol = precision::confusion();
    let (mut aNewBox, mut anOldBox) = (BoundingBox3::default(), BoundingBox3::default());
    if let Some(aPatch) = thePatch {
        anEval.add_range(aPatch, aTol, &mut aNewBox).unwrap();
        AddSurface::add_range(&anAdaptor, aPatch, aTol, &mut anOldBox).unwrap();
    } else {
        anEval.add(aTol, &mut aNewBox).unwrap();
        AddSurface::add(&anAdaptor, aTol, &mut anOldBox).unwrap();
    }
    (aNewBox, anOldBox)
}

// GeomBndLib_Surface(anOff).AddOptimal over the patch, or the natural bounds when None.
fn optimal_box(theSurf: &Handle<dyn GeomSurface>, thePatch: Option<[f64; 4]>) -> BoundingBox3 {
    let anEval = Surface::from_surface(theSurf).unwrap();
    let mut aBox = BoundingBox3::default();
    match thePatch {
        Some(aPatch) => anEval
            .add_optimal_range(aPatch, precision::confusion(), &mut aBox)
            .unwrap(),
        None => anEval
            .add_optimal(precision::confusion(), &mut aBox)
            .unwrap(),
    }
    aBox
}

const SPHERE_FULL: [f64; 4] = [0.0, 2.0 * PI, -PI / 2.0, PI / 2.0];
const TORUS_FULL: [f64; 4] = [0.0, 2.0 * PI, 0.0, 2.0 * PI];

// =========================================================================
// Offset sphere (positive)
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Sphere_PositiveOffset_Full_CompareWithBndLib() {
    let anOff = offset(sphere(5.0), 2.0);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, SPHERE_FULL, 36, 18);
}

#[test]
fn GeomBndLib_OffsetSurfaceTest__Sphere_PositiveOffset_Patch_CompareWithBndLib() {
    let anOff = offset(sphere(5.0), 2.0);
    let aPatch = [0.0, PI, -PI / 4.0, PI / 4.0];
    let (aNewBox, anOldBox) = boxes(&anOff, Some(aPatch));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, aPatch, 18, 9);
}

// =========================================================================
// Offset sphere (negative)
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Sphere_NegativeOffset_Full_CompareWithBndLib() {
    let anOff = offset(sphere(10.0), -3.0);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, SPHERE_FULL, 36, 18);
}

// =========================================================================
// Offset cylinder
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Cylinder_Full_CompareWithBndLib() {
    let anOff = offset(cylinder(4.0), 1.5);
    let aPatch = [0.0, 2.0 * PI, -5.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anOff, Some(aPatch));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, aPatch, 36, 10);
}

#[test]
fn GeomBndLib_OffsetSurfaceTest__Cylinder_Patch_CompareWithBndLib() {
    let anOff = offset(cylinder(4.0), 1.5);
    let aPatch = [0.0, PI, 0.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anOff, Some(aPatch));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, aPatch, 18, 5);
}

#[test]
fn GeomBndLib_OffsetSurfaceTest__Cylinder_NegativeOffset_CompareWithBndLib() {
    let anOff = offset(cylinder(8.0), -2.0);
    let aPatch = [0.0, 2.0 * PI, 0.0, 10.0];
    let (aNewBox, anOldBox) = boxes(&anOff, Some(aPatch));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, aPatch, 36, 10);
}

// =========================================================================
// Offset plane
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Plane_PositiveOffset_CompareWithBndLib() {
    let anOff = offset(plane(), 3.0);
    let aPatch = [-5.0, 5.0, -5.0, 5.0];
    let (aNewBox, anOldBox) = boxes(&anOff, Some(aPatch));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, aPatch, 10, 10);
}

#[test]
fn GeomBndLib_OffsetSurfaceTest__Plane_PositiveOffset_Patch_IsNotArtificiallyThickenedInZ() {
    let anOff = offset(plane(), 3.0);

    let mut aBox = BoundingBox3::default();
    Surface::from_surface(&anOff)
        .unwrap()
        .add_range([-4.0, 6.0, -2.0, 7.0], precision::confusion(), &mut aBox)
        .unwrap();

    let [_aXmin, _aXmax, _aYmin, _aYmax, aZmin, aZmax] = get3(&aBox);

    let aTol = precision::confusion();
    EXPECT_NEAR(aZmin, 3.0 - aTol, 1e-12);
    EXPECT_NEAR(aZmax, 3.0 + aTol, 1e-12);
}

#[test]
fn GeomBndLib_OffsetSurfaceTest__Plane_NegativeOffset_CompareWithBndLib() {
    let anOff = offset(plane(), -5.0);
    let aPatch = [-10.0, 10.0, -10.0, 10.0];
    let (aNewBox, anOldBox) = boxes(&anOff, Some(aPatch));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, aPatch, 10, 10);
}

// =========================================================================
// Offset torus
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Torus_PositiveOffset_CompareWithBndLib() {
    let anOff = offset(torus(), 1.0);
    let (aNewBox, anOldBox) = boxes(&anOff, Some(TORUS_FULL));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, TORUS_FULL, 36, 36);
}

#[test]
fn GeomBndLib_OffsetSurfaceTest__Torus_Patch_CompareWithBndLib() {
    let anOff = offset(torus(), 1.0);
    let aPatch = [0.0, PI, 0.0, PI];
    let (aNewBox, anOldBox) = boxes(&anOff, Some(aPatch));
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, aPatch, 18, 18);
}

// =========================================================================
// BoxOptimal - plane (analytic equivalent path)
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Plane_BoxOptimal_TighterThanBox() {
    let anOff = offset(plane(), 3.0);
    let aPatch = [-5.0, 5.0, -5.0, 5.0];

    let (aFastBox, _) = boxes(&anOff, Some(aPatch));
    let aOptBox = optimal_box(&anOff, Some(aPatch));

    // Offset plane is a plane - BoxOptimal should be at least as tight as Box.
    ExpectNoLarger(&aOptBox, &aFastBox, precision::confusion());
    ExpectContainsOffsetSurface(&aOptBox, &anOff, aPatch, 10, 10);
}

// =========================================================================
// BoxOptimal - torus (PSO fallback path)
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Torus_BoxOptimal_ContainsSurface() {
    let anOff = offset(torus(), 1.0);

    let (aFastBox, _) = boxes(&anOff, Some(TORUS_FULL));
    let aOptBox = optimal_box(&anOff, Some(TORUS_FULL));

    // PSO optimal should not exceed the fast box and must contain the actual surface.
    ExpectNoLarger(&aOptBox, &aFastBox, precision::confusion());
    ExpectContainsOffsetSurface(&aOptBox, &anOff, TORUS_FULL, 36, 36);
}

// =========================================================================
// BoxOptimal - sphere (analytic equivalent: offset sphere -> sphere)
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Sphere_BoxOptimal_ContainsSurface() {
    let anOff = offset(sphere(5.0), 2.0);

    let (aFastBox, _) = boxes(&anOff, None);
    let aOptBox = optimal_box(&anOff, None);

    // Analytic path: offset sphere -> sphere, so BoxOptimal should be at least as tight.
    ExpectNoLarger(&aOptBox, &aFastBox, precision::confusion());
    ExpectContainsOffsetSurface(&aOptBox, &anOff, SPHERE_FULL, 36, 18);
}

// =========================================================================
// Large offset
// =========================================================================

#[test]
fn GeomBndLib_OffsetSurfaceTest__Sphere_LargeOffset_CompareWithBndLib() {
    let anOff = offset(sphere(2.0), 20.0);
    let (aNewBox, anOldBox) = boxes(&anOff, None);
    ExpectNoLarger(&aNewBox, &anOldBox, precision::confusion());
    ExpectContainsOffsetSurface(&aNewBox, &anOff, SPHERE_FULL, 36, 18);
}
