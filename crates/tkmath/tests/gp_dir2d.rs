//! Complete legacy planar primitive suite.
#![allow(non_snake_case, unused_mut, unused_assignments, unused_imports)]
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2, Xy};
#[test]
fn gp_Dir2dTest__CoordinateConstructor() {
    let mut aDir = Direction2::new(1.0, 0.0).unwrap();
    assert!((aDir.x() - 1.0).abs() <= 1e-7);
    assert!((aDir.y() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__XYConstructor() {
    let mut aXY = Xy::new(0.0, 3.0);
    let mut aDir = Direction2::from_xy(aXY).unwrap();
    assert!((aDir.x() - 0.0).abs() <= 1e-7);
    assert!((aDir.y() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__Vec2dConstructor() {
    let mut aVec = Vector2::new(3.0, 0.0);
    let mut aDir = Direction2::from_vector(aVec).unwrap();
    assert!((aDir.x() - 1.0).abs() <= 1e-7);
    assert!((aDir.y() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__Angle() {
    let mut aD1 = Direction2::new(1.0, 0.0).unwrap();
    let mut aD2 = Direction2::new(0.0, 1.0).unwrap();
    assert!((aD1.angle(aD2) - std::f64::consts::FRAC_PI_2).abs() <= 1e-12);
}

#[test]
fn gp_Dir2dTest__IsEqual() {
    let mut aD1 = Direction2::new(1.0, 0.0).unwrap();
    let mut aD2 = Direction2::new(1.0, 0.0).unwrap();
    assert!(aD1.is_equal(aD2, 1e-12));
}

#[test]
fn gp_Dir2dTest__IsNormal() {
    let mut aD1 = Direction2::new(1.0, 0.0).unwrap();
    let mut aD2 = Direction2::new(0.0, 1.0).unwrap();
    assert!(aD1.is_normal(aD2, 1e-12));
}

#[test]
fn gp_Dir2dTest__IsOpposite() {
    let mut aD1 = Direction2::new(1.0, 0.0).unwrap();
    let mut aD2 = Direction2::new(-1.0, 0.0).unwrap();
    assert!(aD1.is_opposite(aD2, 1e-12));
}

#[test]
fn gp_Dir2dTest__IsParallel() {
    let mut aD1 = Direction2::new(1.0, 0.0).unwrap();
    let mut aD2 = Direction2::new(-1.0, 0.0).unwrap();
    assert!(aD1.is_parallel(aD2, 1e-12));
}

#[test]
fn gp_Dir2dTest__Dot() {
    let mut aD1 = Direction2::new(1.0, 0.0).unwrap();
    let mut aD2 = Direction2::new(0.0, 1.0).unwrap();
    assert!((aD1.dot(aD2) - 0.0).abs() <= 1e-7);

    let mut aD3 = Direction2::new(1.0, 0.0).unwrap();
    assert!((aD1.dot(aD3) - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__Crossed() {
    let mut aD1 = Direction2::new(1.0, 0.0).unwrap();
    let mut aD2 = Direction2::new(0.0, 1.0).unwrap();
    // D1.X*D2.Y - D1.Y*D2.X = 1*1 - 0*0 = 1
    assert!((aD1.crossed(aD2) - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__Rotate() {
    let mut aDir = Direction2::new(1.0, 0.0).unwrap();
    let mut aRotated = aDir.rotated(std::f64::consts::FRAC_PI_2);
    assert!((aRotated.x() - 0.0).abs() <= 1e-7);
    assert!((aRotated.y() - 1.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__Mirror() {
    let mut aDir = Direction2::new(1.0, 0.0).unwrap();
    // Mirror through the Y axis
    let mut anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let mut aMirrored = aDir.mirrored_axis(anAxis);
    // Mirroring (1,0) through axis with direction (0,1):
    // d_mirror = 2*(d.axis_dir)*axis_dir - d = 2*0*(0,1) - (1,0) = (-1,0)
    assert!((aMirrored.x() - -1.0).abs() <= 1e-7);
    assert!((aMirrored.y() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__Transform() {
    let mut aDir = Direction2::new(1.0, 0.0).unwrap();
    let mut aTrsf = Transform2::default();
    aTrsf = Transform2::rotation(Point2::new(0.0, 0.0), std::f64::consts::PI);
    let mut aTransformed = aDir.transformed(aTrsf);
    assert!((aTransformed.x() - -1.0).abs() <= 1e-7);
    assert!((aTransformed.y() - 0.0).abs() <= 1e-7);
}

#[test]
fn gp_Dir2dTest__ZeroMagnitudeThrows() {
    assert_eq!(
        Direction2::new(0.0, 0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}
