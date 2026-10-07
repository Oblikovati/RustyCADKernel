//! Direct translation of Graphic3d_BndBox_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3, dz, origin};
use rustycad_tkservice::graphic3d::Graphic3dBndBox3d;

fn v(x: f64, y: f64, z: f64) -> Vec3<f64> {
    Vec3::new(x, y, z)
}

/// `EXPECT_DOUBLE_EQ` of both corners.
fn assert_corners(bounds: &Graphic3dBndBox3d, min: [f64; 3], max: [f64; 3]) {
    let (lo, hi) = (bounds.corner_min(), bounds.corner_max());
    assert_eq!([lo.x, lo.y, lo.z], min);
    assert_eq!([hi.x, hi.y, hi.z], max);
}

/// The 4x4 matrix of a transformation (`gp_Trsf::GetMat4`).
fn mat4_of(transform: &Transform3) -> Mat4<f64> {
    let mut matrix = Mat4::identity();
    transform.get_mat4(&mut matrix);
    matrix
}

#[test]
fn Graphic3d_BndBox3dTest__DefaultConstructor() {
    assert!(!Graphic3dBndBox3d::default().is_valid());
}

#[test]
fn Graphic3d_BndBox3dTest__PointConstructor() {
    let bounds = Graphic3dBndBox3d::from_point(v(1.0, 2.0, 3.0));
    assert!(bounds.is_valid());
    assert_corners(&bounds, [1.0, 2.0, 3.0], [1.0, 2.0, 3.0]);
}

#[test]
fn Graphic3d_BndBox3dTest__PointsConstructor() {
    let bounds = Graphic3dBndBox3d::from_corners(v(1.0, 2.0, 3.0), v(4.0, 5.0, 6.0));
    assert!(bounds.is_valid());
    assert_corners(&bounds, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
}

#[test]
fn Graphic3d_BndBox3dTest__AddPoint() {
    let mut bounds = Graphic3dBndBox3d::default();
    bounds.add(&v(1.5, 2.5, 3.5));
    assert!(bounds.is_valid());
    assert_corners(&bounds, [1.5, 2.5, 3.5], [1.5, 2.5, 3.5]);
}

#[test]
fn Graphic3d_BndBox3dTest__CombineBoxes() {
    let mut first = Graphic3dBndBox3d::from_corners(v(1.0, 2.0, 3.0), v(4.0, 5.0, 6.0));
    let second = Graphic3dBndBox3d::from_corners(v(2.0, 0.0, 0.0), v(6.0, 3.0, 6.0));
    first.combine(&second);
    assert!(first.is_valid());
    assert_corners(&first, [1.0, 0.0, 0.0], [6.0, 5.0, 6.0]);
}

#[test]
fn Graphic3d_BndBox3dTest__BoxSize() {
    let size = Graphic3dBndBox3d::from_corners(v(0.5, -2.0, -3.0), v(3.5, 1.0, 3.0)).size();
    assert_eq!([size.x, size.y, size.z], [3.0, 3.0, 6.0]);
}

#[test]
fn Graphic3d_BndBox3dTest__BoxCenter() {
    let center = Graphic3dBndBox3d::from_corners(v(-4.0, -4.0, -4.0), v(4.0, 4.0, 4.0)).center();
    assert_eq!([center.x, center.y, center.z], [0.0, 0.0, 0.0]);
}

#[test]
fn Graphic3d_BndBox3dTest__BoxArea() {
    assert_eq!(
        Graphic3dBndBox3d::from_corners(v(0.0, 0.0, 0.0), v(2.0, 3.0, 4.0)).area(),
        52.0
    );
}

#[test]
fn Graphic3d_BndBox3dTest__TransformationIdentity() {
    let mut bounds = Graphic3dBndBox3d::from_corners(v(0.0, 0.0, 0.0), v(4.0, 5.0, 6.0));
    bounds.transform(&Mat4::identity());
    assert!(bounds.is_valid());
    assert_corners(&bounds, [0.0, 0.0, 0.0], [4.0, 5.0, 6.0]);
}

#[test]
fn Graphic3d_BndBox3dTest__TransformationTranslation() {
    let mut bounds = Graphic3dBndBox3d::from_corners(v(0.0, 0.0, 0.0), v(1.0, 1.0, 1.0));
    let mut translation = Transform3::default();
    translation.set_translation(Vector3::new(2.0, 3.0, 4.0));
    bounds.transform(&mat4_of(&translation));
    assert!(bounds.is_valid());
    assert_corners(&bounds, [2.0, 3.0, 4.0], [3.0, 4.0, 5.0]);
}

#[test]
fn Graphic3d_BndBox3dTest__TransformationScale() {
    let mut bounds = Graphic3dBndBox3d::from_corners(v(1.0, 1.0, 1.0), v(2.0, 2.0, 2.0));
    let mut scale = Transform3::default();
    scale.set_scale(Point3::new(0.0, 0.0, 0.0), 2.0).unwrap();
    bounds.transform(&mat4_of(&scale));
    assert!(bounds.is_valid());
    assert_corners(&bounds, [2.0, 2.0, 2.0], [4.0, 4.0, 4.0]);
}

/// Each coordinate within `tolerance` of the expected one.
fn assert_near(actual: &Vec3<f64>, expected: [f64; 3], tolerance: f64) {
    for (a, e) in [actual.x, actual.y, actual.z].into_iter().zip(expected) {
        assert!((a - e).abs() < tolerance, "{a} != {e}");
    }
}

#[test]
fn Graphic3d_BndBox3dTest__TransformationRotation() {
    let mut bounds = Graphic3dBndBox3d::from_corners(v(1.0, 0.0, 0.0), v(2.0, 1.0, 1.0));
    let mut rotation = Transform3::default();
    rotation
        .set_rotation(Axis1::new(origin(), dz()), std::f64::consts::PI / 2.0)
        .unwrap();
    bounds.transform(&mat4_of(&rotation));
    assert!(bounds.is_valid());
    assert_near(
        bounds.corner_min(),
        [-1.0, 1.0, 0.0],
        precision::confusion(),
    );
    assert_near(bounds.corner_max(), [0.0, 2.0, 1.0], precision::confusion());
}

#[test]
fn Graphic3d_BndBox3dTest__TransformationComposed() {
    let min = v(-1.062999963760376, -1.062999963760376, -1.1150000095367432);
    let max = v(1.059000015258789, 1.062999963760376, 0.0);
    let axis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.6220217, 0.6836324, -0.3817536).unwrap(),
    );
    let mut transform = Transform3::default();
    transform.set_rotation(axis, 3.4186892).unwrap();
    transform.set_scale_factor(25.4).unwrap();
    transform.set_translation_part(Vector3::new(
        -88.76312074860142,
        51.79953807026674,
        -65.57836431443693,
    ));
    let mut bounds = Graphic3dBndBox3d::from_corners(min, max);
    bounds.transform(&mat4_of(&transform));
    assert_near(
        bounds.corner_min(),
        [-113.92301586642091, 25.240619264201865, -91.49744953097915],
        0.00001,
    );
    assert_near(
        bounds.corner_max(),
        [-45.09248805040103, 87.9443412035472, -20.487612688573407],
        0.00001,
    );
}

#[test]
fn Graphic3d_BndBox3dTest__TransformationInvalidBox() {
    let mut bounds = Graphic3dBndBox3d::default();
    let mut rotation = Transform3::default();
    rotation
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.0, 0.0), dz()),
            std::f64::consts::PI / 4.0,
        )
        .unwrap();
    bounds.transform(&mat4_of(&rotation));
    assert!(!bounds.is_valid());
}
