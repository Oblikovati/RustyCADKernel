//! Direct translation of Geom_Circle_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomGeometry};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Transform3};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

// gp_Ax2(P, gp_Dir(0, 0, 1)): the frame with the automatic X direction.
fn z_frame(center: Point3) -> Axis2 {
    Axis2::new(center, Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap()
}

#[test]
fn Geom_CircleTest__Constructor() {
    let anAxis = z_frame(Point3::new(0.0, 0.0, 0.0));
    let aCircle = Handle::new(GeomCircle::new(anAxis, 5.0).unwrap());
    assert!(!aCircle.is_null());
    EXPECT_NEAR(aCircle.radius(), 5.0, precision::confusion());
}

#[test]
fn Geom_CircleTest__Center() {
    let aCenter = Point3::new(1.0, 2.0, 3.0);
    let anAxis = z_frame(aCenter);
    let aCircle = Handle::new(GeomCircle::new(anAxis, 5.0).unwrap());
    assert!(
        aCircle
            .circ()
            .unwrap()
            .location()
            .is_equal(&aCenter, precision::confusion())
    );
}

#[test]
fn Geom_CircleTest__D0Evaluation() {
    let anAxis = z_frame(Point3::new(0.0, 0.0, 0.0));
    let aCircle = Handle::new(GeomCircle::new(anAxis, 1.0).unwrap());

    // At parameter 0: point should be (1,0,0) for unit circle in XY plane
    let aPnt: Point3 = aCircle.d0(0.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aPnt.z(), 0.0, precision::confusion());

    // At parameter PI/2: point should be (0,1,0)
    let aPnt: Point3 = aCircle.d0(FRAC_PI_2).unwrap();
    EXPECT_NEAR(aPnt.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 1.0, precision::confusion());
    EXPECT_NEAR(aPnt.z(), 0.0, precision::confusion());

    // At parameter PI: point should be (-1,0,0)
    let aPnt: Point3 = aCircle.d0(PI).unwrap();
    EXPECT_NEAR(aPnt.x(), -1.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 0.0, precision::confusion());

    // At parameter 3*PI/2: point should be (0,-1,0)
    let aPnt: Point3 = aCircle.d0(3.0 * FRAC_PI_2).unwrap();
    EXPECT_NEAR(aPnt.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), -1.0, precision::confusion());
}

#[test]
fn Geom_CircleTest__D1Evaluation() {
    let anAxis = z_frame(Point3::new(0.0, 0.0, 0.0));
    let aCircle = Handle::new(GeomCircle::new(anAxis, 1.0).unwrap());

    // At parameter 0: tangent should be (0,1,0) for unit circle
    let (_aPnt, aV1) = aCircle.d1(0.0).unwrap();
    EXPECT_NEAR(aV1.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aV1.y(), 1.0, precision::confusion());
    EXPECT_NEAR(aV1.z(), 0.0, precision::confusion());
}

#[test]
fn Geom_CircleTest__D2Evaluation() {
    let anAxis = z_frame(Point3::new(0.0, 0.0, 0.0));
    let aCircle = Handle::new(GeomCircle::new(anAxis, 1.0).unwrap());

    // At parameter 0: second derivative should be (-1,0,0) pointing toward center
    let (_aPnt, _aV1, aV2) = aCircle.d2(0.0).unwrap();
    EXPECT_NEAR(aV2.x(), -1.0, precision::confusion());
    EXPECT_NEAR(aV2.y(), 0.0, precision::confusion());
    EXPECT_NEAR(aV2.z(), 0.0, precision::confusion());
}

#[test]
fn Geom_CircleTest__Reverse() {
    let anAxis = z_frame(Point3::new(0.0, 0.0, 0.0));
    let aCircle = Handle::new(GeomCircle::new(anAxis, 1.0).unwrap());

    let aParam: f64 = FRAC_PI_4;
    let aReversedParam: f64 = aCircle.reversed_parameter(aParam).unwrap();
    // For a circle: reversed parameter = 2*PI - U
    EXPECT_NEAR(aReversedParam, 2.0 * PI - aParam, precision::confusion());
}

#[test]
fn Geom_CircleTest__Transform() {
    let anAxis = z_frame(Point3::new(0.0, 0.0, 0.0));
    let aCircle = Handle::new(GeomCircle::new(anAxis, 5.0).unwrap());

    let mut aTrsf = Transform3::default();
    aTrsf.set_scale(Point3::new(0.0, 0.0, 0.0), 2.0).unwrap();
    aCircle.transform(aTrsf).unwrap();
    EXPECT_NEAR(aCircle.radius(), 10.0, precision::confusion());
}

#[test]
fn Geom_CircleTest__Copy() {
    let anAxis = z_frame(Point3::new(1.0, 2.0, 3.0));
    let aCircle = Handle::new(GeomCircle::new(anAxis, 7.0).unwrap());

    let aCopy: Handle<dyn GeomGeometry> = aCircle.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyCircle: Handle<GeomCircle> = aCopy.cast();
    assert!(!aCopyCircle.is_null());
    EXPECT_NEAR(aCopyCircle.radius(), 7.0, precision::confusion());
    assert!(
        aCopyCircle
            .circ()
            .unwrap()
            .location()
            .is_equal(&Point3::new(1.0, 2.0, 3.0), precision::confusion())
    );
}
