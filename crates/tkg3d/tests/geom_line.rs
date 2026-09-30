//! Direct translation of Geom_Line_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomGeometry, GeomLine};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

#[test]
fn Geom_LineTest__ConstructFromAx1() {
    let anAx1 = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aLine = Handle::new(GeomLine::new(anAx1));
    assert!(!aLine.is_null());

    let aLoc: Point3 = aLine.lin().location();
    assert!(aLoc.is_equal(&Point3::new(0.0, 0.0, 0.0), precision::confusion()));
}

#[test]
fn Geom_LineTest__ConstructFromPointAndDir() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(1.0, 2.0, 3.0),
        dir(0.0, 0.0, 1.0),
    ));
    assert!(!aLine.is_null());

    let aLoc: Point3 = aLine.lin().location();
    assert!(aLoc.is_equal(&Point3::new(1.0, 2.0, 3.0), precision::confusion()));
    assert!(
        aLine
            .lin()
            .direction()
            .is_equal(&dir(0.0, 0.0, 1.0), precision::angular())
    );
}

#[test]
fn Geom_LineTest__D0Evaluation() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(1.0, 0.0, 0.0),
    ));
    let aPnt: Point3 = aLine.d0(5.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aPnt.z(), 0.0, precision::confusion());
}

#[test]
fn Geom_LineTest__D1Evaluation() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(0.0, 1.0, 0.0),
    ));
    let (aPnt, aV1) = aLine.d1(3.0).unwrap();
    EXPECT_NEAR(aPnt.y(), 3.0, precision::confusion());
    // First derivative is the direction vector (constant for a line)
    EXPECT_NEAR(aV1.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aV1.y(), 1.0, precision::confusion());
    EXPECT_NEAR(aV1.z(), 0.0, precision::confusion());
}

#[test]
fn Geom_LineTest__D2Evaluation() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(1.0, 0.0, 0.0),
    ));
    let (_aPnt, _aV1, aV2) = aLine.d2(1.0).unwrap();
    // Second derivative of a line is zero
    EXPECT_NEAR(aV2.magnitude(), 0.0, precision::confusion());
}

#[test]
fn Geom_LineTest__InfiniteParameters() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(1.0, 0.0, 0.0),
    ));
    assert_eq!(aLine.first_parameter(), -precision::infinite());
    assert_eq!(aLine.last_parameter(), precision::infinite());
}

#[test]
fn Geom_LineTest__Reverse() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(1.0, 0.0, 0.0),
    ));
    aLine.reverse().unwrap();
    assert!(
        aLine
            .lin()
            .direction()
            .is_equal(&dir(-1.0, 0.0, 0.0), precision::angular())
    );
}

#[test]
fn Geom_LineTest__Reversed() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(1.0, 0.0, 0.0),
    ));
    let aReversed: Handle<dyn GeomCurve> = aLine.reversed().unwrap();
    assert!(!aReversed.is_null());
    let aRevLine: Handle<GeomLine> = aReversed.cast();
    assert!(!aRevLine.is_null());
    assert!(
        aRevLine
            .lin()
            .direction()
            .is_equal(&dir(-1.0, 0.0, 0.0), precision::angular())
    );
}

#[test]
fn Geom_LineTest__Transform() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir(1.0, 0.0, 0.0),
    ));
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 0.0, 5.0));
    aLine.transform(aTrsf).unwrap();
    EXPECT_NEAR(aLine.lin().location().z(), 5.0, precision::confusion());
}

#[test]
fn Geom_LineTest__Copy() {
    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(1.0, 2.0, 3.0),
        dir(0.0, 0.0, 1.0),
    ));
    let aCopy: Handle<dyn GeomGeometry> = aLine.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyLine: Handle<GeomLine> = aCopy.cast();
    assert!(!aCopyLine.is_null());
    assert!(
        aCopyLine
            .lin()
            .location()
            .is_equal(&Point3::new(1.0, 2.0, 3.0), precision::confusion())
    );
}
