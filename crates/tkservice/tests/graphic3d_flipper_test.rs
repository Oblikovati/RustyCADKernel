//! Direct translation of Graphic3d_Flipper_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkernel::precision;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::graphic3d::Graphic3dFlipper;

fn matrices_near(lhs: &Mat4<f64>, rhs: &Mat4<f64>, tolerance: f64) -> bool {
    (0..4).all(|c| (0..4).all(|r| (lhs.get_value(r, c) - rhs.get_value(r, c)).abs() <= tolerance))
}

fn axis(location: [f64; 3], normal: [f64; 3], x: [f64; 3]) -> Axis2 {
    let d = |v: [f64; 3]| Direction3::new(v[0], v[1], v[2]).unwrap();
    Axis2::with_x_direction(
        Point3::new(location[0], location[1], location[2]),
        d(normal),
        d(x),
    )
    .unwrap()
}

fn default_flipper() -> Graphic3dFlipper {
    Graphic3dFlipper::new(axis([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]))
}

/// The identity with `(0,0)` and `(1,1)` negated: 180 degrees around Z.
fn half_turn_z() -> Mat4<f64> {
    let mut matrix = Mat4::identity();
    matrix.set_value(0, 0, -1.0);
    matrix.set_value(1, 1, -1.0);
    matrix
}

fn assert_box(bounds: &BoundingBox3, expected: [f64; 6]) {
    for (value, wanted) in bounds.get().unwrap().into_iter().zip(expected) {
        assert!(
            (value - wanted).abs() <= precision::confusion(),
            "{value} != {wanted}"
        );
    }
}

fn unit_box() -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    bounds.update_bounds([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    bounds
}

#[test]
fn Graphic3d_FlipperTest__Compute_IdentityWorldView_ReturnsIdentity() {
    assert!(
        default_flipper()
            .compute(&Mat4::<f64>::identity())
            .is_identity()
    );
}

#[test]
fn Graphic3d_FlipperTest__Compute_180AroundZ_IsSelfInverse() {
    let flip = default_flipper().compute(&half_turn_z());
    assert!(!flip.is_identity());
    assert!(matrices_near(
        &(flip * flip),
        &Mat4::identity(),
        precision::confusion()
    ));
}

#[test]
fn Graphic3d_FlipperTest__Apply_NoFlip_BoxUnchanged() {
    let mut bounds = unit_box();
    default_flipper()
        .apply_bnd(&Mat4::identity(), &mut bounds)
        .unwrap();
    assert_box(&bounds, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn Graphic3d_FlipperTest__Apply_VoidBox_RemainsVoid() {
    let mut bounds = BoundingBox3::default();
    assert!(bounds.is_void());
    default_flipper()
        .apply_bnd(&half_turn_z(), &mut bounds)
        .unwrap();
    assert!(bounds.is_void());
}

#[test]
fn Graphic3d_FlipperTest__Apply_180AroundZ_Involution() {
    let (flipper, mut bounds) = (default_flipper(), unit_box());
    flipper.apply_bnd(&half_turn_z(), &mut bounds).unwrap();
    flipper.apply_bnd(&half_turn_z(), &mut bounds).unwrap();
    assert_box(&bounds, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn Graphic3d_FlipperTest__Compute_ObjectTransformChangesDecision() {
    let flipper = default_flipper();
    let world_view = Mat4::<f64>::identity();
    assert!(flipper.compute(&world_view).is_identity());
    let model_view = world_view * half_turn_z();
    assert!(!flipper.compute(&model_view).is_identity());
}

#[test]
fn Graphic3d_FlipperTest__SetRefPlane_RoundTrip() {
    let flipper = default_flipper();
    flipper.set_ref_plane(axis([10.0, 20.0, 30.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]));
    let location = flipper.ref_plane().location();
    assert!((location.x() - 10.0).abs() <= precision::confusion());
    assert!((location.y() - 20.0).abs() <= precision::confusion());
    assert!((location.z() - 30.0).abs() <= precision::confusion());
}
