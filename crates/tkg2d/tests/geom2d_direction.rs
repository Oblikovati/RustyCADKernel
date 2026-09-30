//! Direct translation of Geom2d_Direction_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//!
//! The pinned RelWithDebInfo oracle does not define `No_Exception`, so the
//! `#ifndef No_Exception` branches are the ones translated.
#![allow(non_snake_case)]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dDirection, Geom2dGeometry, Geom2dVector, Geom2dVectorWithMagnitude,
};
use rustycad_tkmath::gp::{Direction2, Point2, Transform2};
use std::f64::consts::PI;

fn direction(x: f64, y: f64) -> Handle<Geom2dDirection> {
    Handle::new(Geom2dDirection::new(x, y).unwrap())
}

#[test]
fn Geom2d_DirectionTest__ConstructFromCoords() {
    let aDir = direction(3.0, 4.0);
    // Should be normalized
    EXPECT_NEAR(aDir.magnitude(), 1.0, precision::confusion());
    EXPECT_NEAR(aDir.x(), 0.6, precision::confusion());
    EXPECT_NEAR(aDir.y(), 0.8, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__ConstructFromDir2d() {
    let aDir = Handle::new(Geom2dDirection::from_dir2d(
        Direction2::new(0.0, 1.0).unwrap(),
    ));
    EXPECT_NEAR(aDir.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aDir.y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__ConstructFromZero_Throws() {
    let aResult = Geom2dDirection::new(0.0, 0.0);
    assert_eq!(
        aResult.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn Geom2d_DirectionTest__MagnitudeAlwaysOne() {
    let aDir = direction(100.0, 200.0);
    EXPECT_NEAR(aDir.magnitude(), 1.0, precision::confusion());
    EXPECT_NEAR(aDir.square_magnitude(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__SetCoord() {
    let aDir = direction(1.0, 0.0);
    aDir.set_coord(0.0, 5.0).unwrap();
    EXPECT_NEAR(aDir.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aDir.y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__SetCoord_Zero_Throws() {
    let aDir = direction(1.0, 0.0);
    let aResult = aDir.set_coord(0.0, 0.0);
    assert_eq!(
        aResult.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn Geom2d_DirectionTest__Dir2d() {
    let aDir = direction(1.0, 0.0);
    let aGpDir: Direction2 = aDir.dir2d().unwrap();
    EXPECT_NEAR(aGpDir.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aGpDir.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__Crossed() {
    let aDirX = direction(1.0, 0.0);
    let aDirY = direction(0.0, 1.0);

    // (1,0) x (0,1) = 1
    EXPECT_NEAR(aDirX.crossed(&*aDirY), 1.0, precision::confusion());
    // (0,1) x (1,0) = -1
    EXPECT_NEAR(aDirY.crossed(&*aDirX), -1.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__Angle() {
    let aDirX = direction(1.0, 0.0);
    let aDirY = direction(0.0, 1.0);

    EXPECT_NEAR(
        aDirX.angle(&*aDirY).unwrap(),
        PI / 2.0,
        precision::confusion(),
    );
}

#[test]
fn Geom2d_DirectionTest__Angle_OppositeDirections() {
    let aDirPos = direction(1.0, 0.0);
    let aDirNeg = direction(-1.0, 0.0);

    EXPECT_NEAR(
        aDirPos.angle(&*aDirNeg).unwrap().abs(),
        PI,
        precision::confusion(),
    );
}

#[test]
fn Geom2d_DirectionTest__Dot() {
    let aDirX = direction(1.0, 0.0);
    let aDirY = direction(0.0, 1.0);

    EXPECT_NEAR(aDirX.dot(&*aDirY), 0.0, precision::confusion());
    EXPECT_NEAR(aDirX.dot(&*aDirX), 1.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__Reverse() {
    let aDir = direction(1.0, 0.0);
    aDir.reverse();
    EXPECT_NEAR(aDir.x(), -1.0, precision::confusion());
    EXPECT_NEAR(aDir.y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__CrossedWithVectorWithMagnitude() {
    let aDir = direction(1.0, 0.0);
    let aVec = Handle::new(Geom2dVectorWithMagnitude::new(0.0, 5.0));

    EXPECT_NEAR(aDir.crossed(&*aVec), 5.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__Copy() {
    let aDir = direction(1.0, 0.0);
    let aCopyGeom: Handle<dyn Geom2dGeometry> = aDir.copy().unwrap();
    let aCopy: Handle<Geom2dDirection> = aCopyGeom.cast::<Geom2dDirection>();

    assert!(!aCopy.is_null());
    EXPECT_NEAR(aCopy.x(), 1.0, precision::confusion());

    aCopy.set_coord(0.0, 1.0).unwrap();
    EXPECT_NEAR(aDir.x(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_DirectionTest__Transform_Rotation() {
    let mut aTrsf = Transform2::default();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI / 2.0);

    let aDir = direction(1.0, 0.0);
    aDir.transform(aTrsf).unwrap();

    EXPECT_NEAR(aDir.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aDir.y(), 1.0, precision::confusion());
    EXPECT_NEAR(aDir.magnitude(), 1.0, precision::confusion());
}
