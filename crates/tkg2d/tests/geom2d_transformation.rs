//! Direct translation of Geom2d_Transformation_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dTransformation;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, TransformForm, Vector2};
use std::f64::consts::PI;

fn identity() -> Handle<Geom2dTransformation> {
    Handle::new(Geom2dTransformation::default())
}

#[test]
fn Geom2d_TransformationTest__DefaultConstructor_Identity() {
    let aTrsf = identity();

    assert_eq!(aTrsf.form(), TransformForm::Identity);
    EXPECT_NEAR(aTrsf.scale_factor(), 1.0, precision::confusion());
    assert!(!aTrsf.is_negative());
}

#[test]
fn Geom2d_TransformationTest__ConstructFromTrsf2d() {
    let mut aGpTrsf = Transform2::default();
    aGpTrsf.set_translation(Vector2::new(3.0, 4.0));

    let aTrsf = Handle::new(Geom2dTransformation::new(aGpTrsf));
    assert_eq!(aTrsf.form(), TransformForm::Translation);
}

#[test]
fn Geom2d_TransformationTest__SetTranslation_Vector() {
    let aTrsf = identity();
    aTrsf.set_translation(Vector2::new(5.0, 10.0));

    assert_eq!(aTrsf.form(), TransformForm::Translation);

    let (aX, aY) = aTrsf.transforms(1.0, 2.0);
    EXPECT_NEAR(aX, 6.0, precision::confusion());
    EXPECT_NEAR(aY, 12.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetTranslation_TwoPoints() {
    let aTrsf = identity();
    aTrsf.set_translation_between(Point2::new(1.0, 1.0), Point2::new(4.0, 5.0));

    let (aX, aY) = aTrsf.transforms(0.0, 0.0);
    EXPECT_NEAR(aX, 3.0, precision::confusion());
    EXPECT_NEAR(aY, 4.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetRotation() {
    let aTrsf = identity();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI / 2.0);

    assert_eq!(aTrsf.form(), TransformForm::Rotation);

    let (aX, aY) = aTrsf.transforms(1.0, 0.0);
    EXPECT_NEAR(aX, 0.0, precision::confusion());
    EXPECT_NEAR(aY, 1.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetRotation_180Degrees() {
    let aTrsf = identity();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI);

    let (aX, aY) = aTrsf.transforms(3.0, 4.0);
    EXPECT_NEAR(aX, -3.0, precision::confusion());
    EXPECT_NEAR(aY, -4.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetScale() {
    let aTrsf = identity();
    aTrsf.set_scale(Point2::new(0.0, 0.0), 2.0);

    assert_eq!(aTrsf.form(), TransformForm::Scale);
    EXPECT_NEAR(aTrsf.scale_factor(), 2.0, precision::confusion());

    let (aX, aY) = aTrsf.transforms(3.0, 4.0);
    EXPECT_NEAR(aX, 6.0, precision::confusion());
    EXPECT_NEAR(aY, 8.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetScale_Negative() {
    let aTrsf = identity();
    aTrsf.set_scale(Point2::new(0.0, 0.0), -1.0);

    // In 2D, IsNegative checks determinant sign; scale(-1) has det = (-1)*(-1) = 1 > 0
    assert!(!aTrsf.is_negative());
    EXPECT_NEAR(aTrsf.scale_factor(), -1.0, precision::confusion());

    let (aX, aY) = aTrsf.transforms(3.0, 4.0);
    EXPECT_NEAR(aX, -3.0, precision::confusion());
    EXPECT_NEAR(aY, -4.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetMirror_Point() {
    let aTrsf = identity();
    aTrsf.set_mirror_point(Point2::new(0.0, 0.0));

    assert_eq!(aTrsf.form(), TransformForm::PointMirror);
    // In 2D, point mirror has det = (-1)*(-1) = 1 > 0, so IsNegative is false
    assert!(!aTrsf.is_negative());

    let (aX, aY) = aTrsf.transforms(3.0, 4.0);
    EXPECT_NEAR(aX, -3.0, precision::confusion());
    EXPECT_NEAR(aY, -4.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetMirror_Axis() {
    // Mirror about X axis
    let aTrsf = identity();
    aTrsf.set_mirror_axis(Axis2d::new(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ));

    assert_eq!(aTrsf.form(), TransformForm::AxisMirror);

    let (aX, aY) = aTrsf.transforms(3.0, 4.0);
    EXPECT_NEAR(aX, 3.0, precision::confusion());
    EXPECT_NEAR(aY, -4.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__Inverted() {
    let aTrsf = identity();
    aTrsf.set_translation(Vector2::new(5.0, 10.0));

    let anInv = aTrsf.inverted().unwrap();

    // Applying both should be identity
    let (aX, aY) = aTrsf.transforms(7.0, 3.0);
    let (aX, aY) = anInv.transforms(aX, aY);
    EXPECT_NEAR(aX, 7.0, precision::confusion());
    EXPECT_NEAR(aY, 3.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__Multiplied() {
    // Translation + Rotation
    let aTrans = identity();
    aTrans.set_translation(Vector2::new(1.0, 0.0));

    let aRot = identity();
    aRot.set_rotation(Point2::new(0.0, 0.0), PI / 2.0);

    // aRot * aTrans: first translate, then rotate
    let aComposed = aRot.multiplied(&aTrans);

    let (aX, aY) = aComposed.transforms(0.0, 0.0);
    // (0,0) -> translate to (1,0) -> rotate 90deg to (0,1)
    EXPECT_NEAR(aX, 0.0, precision::confusion());
    EXPECT_NEAR(aY, 1.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__Power_Zero_IsIdentity() {
    let aTrsf = identity();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI / 4.0);
    aTrsf.power(0).unwrap();

    assert_eq!(aTrsf.form(), TransformForm::Identity);
}

#[test]
fn Geom2d_TransformationTest__Power_Positive() {
    let aTrsf = identity();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI / 2.0);
    aTrsf.power(2).unwrap(); // Two 90-degree rotations = 180 degrees

    let (aX, aY) = aTrsf.transforms(1.0, 0.0);
    EXPECT_NEAR(aX, -1.0, precision::confusion());
    EXPECT_NEAR(aY, 0.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__Value_Matrix() {
    let aTrsf = identity();
    // Identity matrix values
    EXPECT_NEAR(aTrsf.value(1, 1).unwrap(), 1.0, precision::confusion());
    EXPECT_NEAR(aTrsf.value(1, 2).unwrap(), 0.0, precision::confusion());
    EXPECT_NEAR(aTrsf.value(2, 1).unwrap(), 0.0, precision::confusion());
    EXPECT_NEAR(aTrsf.value(2, 2).unwrap(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__Trsf2d_RoundTrip() {
    let mut aGpTrsf = Transform2::default();
    aGpTrsf.set_rotation(Point2::new(1.0, 2.0), PI / 3.0);

    let aTrsf = Handle::new(Geom2dTransformation::new(aGpTrsf));
    let aBack: Transform2 = aTrsf.trsf2d();

    // Verify the round-trip preserves the transformation
    let (aX1, aY1) = aGpTrsf.transforms_coordinates(5.0, 7.0);
    let (aX2, aY2) = aBack.transforms_coordinates(5.0, 7.0);
    EXPECT_NEAR(aX1, aX2, precision::confusion());
    EXPECT_NEAR(aY1, aY2, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__Copy() {
    let aTrsf = identity();
    aTrsf.set_scale(Point2::new(0.0, 0.0), 3.0);

    let aCopy: Handle<Geom2dTransformation> = aTrsf.copy();
    EXPECT_NEAR(aCopy.scale_factor(), 3.0, precision::confusion());

    // Verify independence
    aCopy.set_scale(Point2::new(0.0, 0.0), 5.0);
    EXPECT_NEAR(aTrsf.scale_factor(), 3.0, precision::confusion());
}

#[test]
fn Geom2d_TransformationTest__SetScale_WithCenter() {
    // Scaling with center at (2, 3) by factor 2
    let aTrsf = identity();
    aTrsf.set_scale(Point2::new(2.0, 3.0), 2.0);

    // The center should remain fixed
    let (aX, aY) = aTrsf.transforms(2.0, 3.0);
    EXPECT_NEAR(aX, 2.0, precision::confusion());
    EXPECT_NEAR(aY, 3.0, precision::confusion());

    // A different point should scale from the center
    let (aX, aY) = aTrsf.transforms(4.0, 3.0);
    EXPECT_NEAR(aX, 6.0, precision::confusion()); // 2 + 2*(4-2) = 6
    EXPECT_NEAR(aY, 3.0, precision::confusion()); // 3 + 2*(3-3) = 3
}

#[test]
fn Geom2d_TransformationTest__SetRotation_WithCenter() {
    // 90-degree rotation about (1, 1)
    let aTrsf = identity();
    aTrsf.set_rotation(Point2::new(1.0, 1.0), PI / 2.0);

    // Center stays fixed
    let (aX, aY) = aTrsf.transforms(1.0, 1.0);
    EXPECT_NEAR(aX, 1.0, precision::confusion());
    EXPECT_NEAR(aY, 1.0, precision::confusion());

    // (2, 1) relative to center is (1, 0), rotated 90 deg is (0, 1), so result is (1, 2)
    let (aX, aY) = aTrsf.transforms(2.0, 1.0);
    EXPECT_NEAR(aX, 1.0, precision::confusion());
    EXPECT_NEAR(aY, 2.0, precision::confusion());
}
