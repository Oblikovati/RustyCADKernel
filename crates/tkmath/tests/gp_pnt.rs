//! All gp_Pnt legacy assertions with the native arithmetic path.
#![allow(non_snake_case, unused_mut, unused_assignments)]
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3, Vector3, Xyz};
#[test]
fn gp_PntTest__DefaultConstructor() {
    let mut aPnt = Point3::default();
    assert_eq!(aPnt.x(), 0.0);
    assert_eq!(aPnt.y(), 0.0);
    assert_eq!(aPnt.z(), 0.0);
}

#[test]
fn gp_PntTest__CoordinateConstructor() {
    let mut aPnt = Point3::new(1.0, 2.0, 3.0);
    assert_eq!(aPnt.x(), 1.0);
    assert_eq!(aPnt.y(), 2.0);
    assert_eq!(aPnt.z(), 3.0);
}

#[test]
fn gp_PntTest__XYZConstructor() {
    let mut aXYZ = Xyz::new(4.0, 5.0, 6.0);
    let mut aPnt = Point3::from(aXYZ);
    assert_eq!(aPnt.x(), 4.0);
    assert_eq!(aPnt.y(), 5.0);
    assert_eq!(aPnt.z(), 6.0);
}

#[test]
fn gp_PntTest__SetCoord() {
    let mut aPnt = Point3::default();
    aPnt.set_x(10.0);
    aPnt.set_y(20.0);
    aPnt.set_z(30.0);
    assert_eq!(aPnt.x(), 10.0);
    assert_eq!(aPnt.y(), 20.0);
    assert_eq!(aPnt.z(), 30.0);

    aPnt.set_coordinates(7.0, 8.0, 9.0);
    assert_eq!(aPnt.x(), 7.0);
    assert_eq!(aPnt.y(), 8.0);
    assert_eq!(aPnt.z(), 9.0);
}

#[test]
fn gp_PntTest__Distance() {
    let mut aP1 = Point3::new(0.0, 0.0, 0.0);
    let mut aP2 = Point3::new(3.0, 4.0, 0.0);
    assert!((aP1.distance(&aP2) - 5.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__SquareDistance() {
    let mut aP1 = Point3::new(1.0, 2.0, 3.0);
    let mut aP2 = Point3::new(4.0, 6.0, 3.0);
    // (4-1)^2 + (6-2)^2 + (3-3)^2 = 9 + 16 + 0 = 25
    assert!((aP1.square_distance(&aP2) - 25.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__IsEqual() {
    let mut aP1 = Point3::new(1.0, 2.0, 3.0);
    let mut aP2 = Point3::new(1.0, 2.0, 3.0);
    let mut aP3 = Point3::new(1.0, 2.0, 4.0);
    assert!(aP1.is_equal(&aP2, 1e-7));
    assert!(!(aP1.is_equal(&aP3, 1e-7)));
}

#[test]
fn gp_PntTest__BaryCenter() {
    let mut aP1 = Point3::new(0.0, 0.0, 0.0);
    let mut aP2 = Point3::new(10.0, 0.0, 0.0);
    // BaryCenter with equal weights: midpoint
    aP1.barycenter(1.0, aP2, 1.0);
    assert!((aP1.x() - 5.0).abs() <= 1e-7);
    assert!((aP1.y() - 0.0).abs() <= 1e-7);
    assert!((aP1.z() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Translate_ByVec() {
    let mut aPnt = Point3::new(1.0, 2.0, 3.0);
    let mut aVec = Vector3::new(10.0, 20.0, 30.0);
    let mut aResult = aPnt.translated(aVec);
    assert!((aResult.x() - 11.0).abs() <= 1e-7);
    assert!((aResult.y() - 22.0).abs() <= 1e-7);
    assert!((aResult.z() - 33.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Translate_ByTwoPoints() {
    let mut aPnt = Point3::new(1.0, 2.0, 3.0);
    let mut aFrom = Point3::new(0.0, 0.0, 0.0);
    let mut aTo = Point3::new(5.0, 5.0, 5.0);
    let mut aResult = aPnt.translated_between(aFrom, aTo);
    assert!((aResult.x() - 6.0).abs() <= 1e-7);
    assert!((aResult.y() - 7.0).abs() <= 1e-7);
    assert!((aResult.z() - 8.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Scale() {
    let mut aPnt = Point3::new(2.0, 4.0, 6.0);
    let mut aCenter = Point3::new(0.0, 0.0, 0.0);
    let mut aResult = aPnt.scaled(aCenter, 2.0);
    assert!((aResult.x() - 4.0).abs() <= 1e-7);
    assert!((aResult.y() - 8.0).abs() <= 1e-7);
    assert!((aResult.z() - 12.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Rotate() {
    let mut aPnt = Point3::new(1.0, 0.0, 0.0);
    let mut anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let mut aResult = aPnt.rotated(anAxis, std::f64::consts::FRAC_PI_2).unwrap();
    assert!((aResult.x() - 0.0).abs() <= 1e-7);
    assert!((aResult.y() - 1.0).abs() <= 1e-7);
    assert!((aResult.z() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Mirror_Point() {
    let mut aPnt = Point3::new(1.0, 0.0, 0.0);
    let mut aCenter = Point3::new(0.0, 0.0, 0.0);
    let mut aResult = aPnt.mirrored_point(aCenter);
    assert!((aResult.x() - -1.0).abs() <= 1e-7);
    assert!((aResult.y() - 0.0).abs() <= 1e-7);
    assert!((aResult.z() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Mirror_Axis() {
    let mut aPnt = Point3::new(1.0, 1.0, 0.0);
    let mut anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let mut aResult = aPnt.mirrored_axis(anAxis);
    assert!((aResult.x() - 1.0).abs() <= 1e-7);
    assert!((aResult.y() - -1.0).abs() <= 1e-7);
    assert!((aResult.z() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Mirror_Plane() {
    let mut aPnt = Point3::new(1.0, 2.0, 3.0);
    // Mirror through XY plane (Z=0)
    let mut anAx2 = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let mut aResult = aPnt.mirrored_plane(anAx2);
    assert!((aResult.x() - 1.0).abs() <= 1e-7);
    assert!((aResult.y() - 2.0).abs() <= 1e-7);
    assert!((aResult.z() - -3.0).abs() <= 1e-7);
}

#[test]
fn gp_PntTest__Transformed() {
    let mut aPnt = Point3::new(1.0, 0.0, 0.0);
    let mut aTrsf = Transform3::default();
    aTrsf = Transform3::translation([0.0, 0.0, 5.0]);
    let mut aResult = aPnt.transformed(aTrsf);
    assert!((aResult.x() - 1.0).abs() <= 1e-7);
    assert!((aResult.y() - 0.0).abs() <= 1e-7);
    assert!((aResult.z() - 5.0).abs() <= 1e-7);
}
