//! Supplemental in-place point APIs and vector failure semantics.
use rustycad_tkmath::gp::{
    Axis1, Axis2, Direction3, Point3, Transform3, TransformForm, Vector3, Xyz,
};
#[test]
fn point_mutators_match_copy_operations() {
    let mut point = Point3::default();
    point.set_coord(1, 1.0).unwrap();
    assert_eq!(point.coord(1).unwrap(), 1.0);
    assert!(point.coord(0).is_err());
    assert!(point.set_coord(4, 9.0).is_err());
    let origin = Point3::default();
    let vector = Vector3::new(2.0, 3.0, 4.0);
    point.translate(vector);
    point.translate_between(origin, Point3::new(1.0, 1.0, 1.0));
    assert_eq!(point, Point3::new(4.0, 4.0, 5.0));
    point.scale(origin, 2.0);
    assert_eq!(point, Point3::new(8.0, 8.0, 10.0));
    let axis = Axis1::default();
    point.rotate(axis, 0.0).unwrap();
    point.mirror_point(origin);
    assert_eq!(point, Point3::new(-8.0, -8.0, -10.0));
    let copied = point.mirrored_axis(axis);
    point.mirror_axis(axis);
    assert_eq!(point, copied);
    let plane = Axis2::default();
    let copied = point.mirrored_plane(plane);
    point.mirror_plane(plane);
    assert_eq!(point, copied);
    let transform = Transform3::rotation(axis, 0.25).unwrap();
    assert_eq!(transform.form(), TransformForm::Rotation);
    assert_eq!(transform.translation_part(), Xyz::default());
}
#[test]
fn vector_normalization_and_angle_errors() {
    let mut zero = Vector3::default();
    assert_eq!(
        zero.normalize().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        zero.angle(Vector3::new(1.0, 0.0, 0.0))
            .unwrap_err()
            .exception_type(),
        "gp_VectorWithNullMagnitude"
    );
    let mut vector = Vector3::new(3.0, 4.0, 0.0);
    vector.normalize().unwrap();
    assert_eq!(vector, Vector3::new(0.6, 0.8, 0.0));
    vector.reverse();
    assert_eq!(vector, Vector3::new(-0.6, -0.8, -0.0));
    let unit = Vector3::from(Direction3::X);
    assert_eq!(unit, Vector3::new(1.0, 0.0, 0.0));
}
