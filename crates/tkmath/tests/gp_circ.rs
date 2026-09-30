//! Complete gp_Circ legacy suite, preserving every assertion.
#![allow(non_snake_case, unused_assignments, unused_imports)]
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Circle3, Direction3, Point3, Transform3, Vector3};
#[test]
fn gp_CircTest__Constructor() {
    let anAxis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCirc = Circle3::new(anAxis, 5.0).unwrap();
    assert!((aCirc.radius() - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_CircTest__Area() {
    let anAxis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCirc = Circle3::new(anAxis, 1.0).unwrap();
    assert!((aCirc.area() - std::f64::consts::PI).abs() <= 1e-7);
}

#[test]
fn gp_CircTest__Length() {
    let anAxis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCirc = Circle3::new(anAxis, 1.0).unwrap();
    assert!((aCirc.length() - 2.0 * std::f64::consts::PI).abs() <= 1e-7);
}

#[test]
fn gp_CircTest__Center() {
    let aCenter = Point3::new(1.0, 2.0, 3.0);
    let anAxis = Axis2::new(aCenter, Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    let aCirc = Circle3::new(anAxis, 5.0).unwrap();
    assert!(aCirc.location().is_equal(&aCenter, 1e-7));
}

#[test]
fn gp_CircTest__Translate() {
    let anAxis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCirc = Circle3::new(anAxis, 5.0).unwrap();
    let aVec = Vector3::new(10.0, 20.0, 30.0);
    let aTranslated = aCirc.translated(aVec);
    assert!((aTranslated.location().x() - 10.0).abs() <= 1e-7);
    assert!((aTranslated.location().y() - 20.0).abs() <= 1e-7);
    assert!((aTranslated.location().z() - 30.0).abs() <= 1e-7);
    assert!((aTranslated.radius() - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_CircTest__Rotate() {
    let anAxis = Axis2::new(
        Point3::new(1.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCirc = Circle3::new(anAxis, 2.0).unwrap();
    let aRotAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let aRotated = aCirc
        .rotated(aRotAxis, std::f64::consts::FRAC_PI_2)
        .unwrap();
    assert!((aRotated.location().x() - 0.0).abs() <= 1e-7);
    assert!((aRotated.location().y() - 1.0).abs() <= 1e-7);
    assert!((aRotated.radius() - 2.0).abs() <= 1e-7);
}

#[test]
fn gp_CircTest__Mirror() {
    let anAxis = Axis2::new(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCirc = Circle3::new(anAxis, 5.0).unwrap();
    let aMirrorPnt = Point3::new(0.0, 0.0, 0.0);
    let aMirrored = aCirc.mirrored_point(aMirrorPnt);
    assert!((aMirrored.location().x() - -1.0).abs() <= 1e-7);
    assert!((aMirrored.location().y() - -2.0).abs() <= 1e-7);
    assert!((aMirrored.location().z() - -3.0).abs() <= 1e-7);
}

#[test]
fn gp_CircTest__Transform() {
    let anAxis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let aCirc = Circle3::new(anAxis, 3.0).unwrap();
    let mut aTrsf = Transform3::default();
    aTrsf = Transform3::scaling(Point3::default(), 2.0).unwrap();
    let aTransformed = aCirc.transformed(aTrsf).unwrap();
    assert!((aTransformed.radius() - 6.0).abs() <= 1e-7);
}
