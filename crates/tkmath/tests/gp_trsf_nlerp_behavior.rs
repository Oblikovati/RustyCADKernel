//! gp_TrsfNLerp: part-wise interpolation of rotation, translation and scale.
use rustycad_tkmath::gp::{
    Axis1, Direction3, Point3, Transform3, TransformForm, TransformNLerp, Vector3, Xyz,
};

fn rotation_about_z(angle: f64) -> Transform3 {
    Transform3::rotation(Axis1::new(Point3::default(), Direction3::Z), angle).unwrap()
}

#[test]
fn ends_within_confusion_return_the_stored_transformations() {
    let start = Transform3::translation_vector(Vector3::new(1.0, 2.0, 3.0));
    let mut end = rotation_about_z(1.0);
    end.set_scale_factor(4.0).unwrap();
    let lerp = TransformNLerp::new(start, end);
    assert_eq!(lerp.interpolate(1e-8).unwrap(), start);
    assert_eq!(lerp.interpolate(1.0 - 1e-8).unwrap(), end);
}

#[test]
fn midpoint_interpolates_each_part_independently() {
    let mut start = Transform3::translation_vector(Vector3::new(2.0, 0.0, 0.0));
    start.set_scale_factor(1.0).unwrap();
    let mut end = rotation_about_z(std::f64::consts::FRAC_PI_2);
    end.set_translation_part(Vector3::new(0.0, 4.0, 0.0));
    end.set_scale_factor(3.0).unwrap();
    let middle = TransformNLerp::new(start, end).interpolate(0.5).unwrap();
    assert_eq!(middle.translation_part(), Xyz::new(1.0, 2.0, 0.0));
    assert_eq!(middle.scale_factor(), 2.0);
    assert_eq!(middle.form(), TransformForm::Compound);
    // NLerp does not renormalize: the half-way quaternion is shorter than unit.
    let (axis, angle) = middle.rotation_axis_angle();
    assert!(
        (angle - std::f64::consts::FRAC_PI_4).abs() <= 1e-15,
        "{angle}"
    );
    assert!((axis.z() - 1.0).abs() <= 1e-15);
}

#[test]
fn an_interpolated_null_scale_keeps_the_native_raise() {
    let mut start = Transform3::default();
    start.set_scale_factor(-1.0).unwrap();
    let mut lerp = TransformNLerp::default();
    lerp.init(start, Transform3::default());
    let failure = lerp.interpolate(0.5).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert_eq!(lerp.interpolate(0.25).unwrap().scale_factor(), -0.5);
}
