//! Supplemental gp_Trsf2d and gp_GTrsf2d behavior pinned against native OCCT.
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Direction2, Direction3, GeneralTransform2, Matrix2, Point2, Point3, Transform2,
    Transform3, TransformForm, Vector2, Xy,
};
use std::f64::consts::FRAC_PI_2;

fn axis(x: f64, y: f64, dx: f64, dy: f64) -> Axis2d {
    Axis2d::new(Point2::new(x, y), Direction2::new(dx, dy).unwrap())
}

#[test]
fn constructors_store_native_forms_and_parts() {
    let mirror = Transform2::mirror_point(Point2::new(1.0, 2.0));
    assert_eq!(mirror.form(), TransformForm::PointMirror);
    assert_eq!(mirror.scale_factor(), -1.0);
    assert_eq!(mirror.translation_part(), Xy::new(2.0, 4.0));
    // IsNegative reads the unscaled matrix, which is the identity for a point mirror.
    assert!(!mirror.is_negative());
    assert!(Transform2::mirror_axis(Axis2d::default()).is_negative());
    let scale = Transform2::scaling(Point2::new(1.0, 1.0), 3.0);
    assert_eq!(scale.form(), TransformForm::Scale);
    assert_eq!(scale.translation_part(), Xy::new(-2.0, -2.0));
    assert_eq!(scale.value(1, 1).unwrap(), 3.0);
    assert_eq!(scale.value(2, 3).unwrap(), -2.0);
    assert!(scale.value(3, 1).is_err());
    let between = Transform2::translation_between(Point2::new(1.0, 1.0), Point2::new(4.0, 5.0));
    assert_eq!(between.translation_part(), Xy::new(3.0, 4.0));
    assert_eq!(
        between,
        Transform2::translation_vector(Vector2::new(3.0, 4.0))
    );
    let mut setter = Transform2::default();
    setter.set_mirror_point(Point2::new(1.0, 2.0));
    assert_eq!(setter, mirror);
    setter.set_scale(Point2::new(1.0, 1.0), 3.0);
    assert_eq!(setter, scale);
    setter.set_translation_between(Point2::new(1.0, 1.0), Point2::new(4.0, 5.0));
    assert_eq!(setter, between);
    setter.set_translation(Vector2::new(3.0, 4.0));
    assert_eq!(setter, between);
    setter.set_mirror_axis(Axis2d::default());
    assert_eq!(setter, Transform2::mirror_axis(Axis2d::default()));
    setter.set_rotation(Point2::default(), 1.0);
    assert_eq!(setter, Transform2::rotation(Point2::default(), 1.0));
}

#[test]
fn transformation_between_axes_maps_frames() {
    let from = axis(1.0, 0.0, 0.0, 1.0);
    let to = axis(0.0, 2.0, 1.0, 0.0);
    let transform = Transform2::transformation_between(from, to);
    assert_eq!(transform.form(), TransformForm::Compound);
    // The origin of `from` is the global point (1, 0), which is (1, -2) in `to`.
    let image = transform.apply_coordinates(Xy::new(0.0, 0.0));
    assert_eq!(image, Xy::new(1.0, -2.0));
    let mut setter = Transform2::default();
    setter.set_transformation_between(from, to);
    assert_eq!(setter, transform);
    setter.set_transformation_to(to);
    assert_eq!(setter, Transform2::transformation_to(to));
    assert_eq!(transform.transforms_coordinates(0.0, 0.0), (1.0, -2.0));
}

#[test]
fn scale_factor_and_translation_part_rederive_forms() {
    let mut transform = Transform2::default();
    transform.set_scale_factor(-1.0);
    assert_eq!(transform.form(), TransformForm::PointMirror);
    transform.set_scale_factor(1.0);
    assert_eq!(transform.form(), TransformForm::Translation);
    transform = Transform2::default();
    transform.set_scale_factor(2.0);
    assert_eq!(transform.form(), TransformForm::Scale);
    transform.set_scale_factor(1.0);
    assert_eq!(transform.form(), TransformForm::Identity);
    transform = Transform2::rotation(Point2::default(), 1.0);
    transform.set_scale_factor(2.0);
    assert_eq!(transform.form(), TransformForm::Compound);
    transform = Transform2::default();
    transform.set_translation_part(Vector2::new(1.0, 0.0));
    assert_eq!(transform.form(), TransformForm::Translation);
    transform.set_translation_part(Vector2::new(0.0, 0.0));
    assert_eq!(transform.form(), TransformForm::Identity);
    transform = Transform2::rotation(Point2::default(), 1.0);
    transform.set_translation_part(Vector2::new(0.0, 0.0));
    assert_eq!(transform.form(), TransformForm::Rotation);
    transform.set_translation_part(Vector2::new(1.0, 0.0));
    assert_eq!(transform.form(), TransformForm::Compound);
}

#[test]
fn set_values_factors_scale_and_rejects_null_determinants() {
    let mut transform = Transform2::default();
    transform.set_values(0.0, -2.0, 5.0, 2.0, 0.0, 6.0).unwrap();
    assert_eq!(transform.form(), TransformForm::Compound);
    assert_eq!(transform.scale_factor(), 2.0);
    assert_eq!(transform.matrix(), Matrix2::new([[0.0, -1.0], [1.0, 0.0]]));
    assert_eq!(transform.translation_part(), Xy::new(5.0, 6.0));
    assert_eq!(transform.rotation_part(), FRAC_PI_2);
    assert_eq!(
        transform.vectorial_part(),
        Matrix2::new([[0.0, -2.0], [2.0, 0.0]])
    );
    let error = transform
        .set_values(1.0, 2.0, 0.0, 2.0, 4.0, 0.0)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(
        transform.scale_factor(),
        2.0,
        "state kept on the determinant raise"
    );
}

#[test]
fn vectorial_part_scales_only_the_diagonal_of_scale_forms() {
    let scale = Transform2::scaling(Point2::default(), 3.0);
    assert_eq!(
        scale.vectorial_part(),
        Matrix2::new([[3.0, 0.0], [0.0, 3.0]])
    );
    let mirror = Transform2::mirror_axis(axis(0.0, 0.0, 0.6, 0.8));
    assert_eq!(mirror.vectorial_part(), mirror.matrix() * -1.0);
}

#[test]
fn inversion_and_powers_follow_each_form() {
    let zero = Transform2::scaling(Point2::default(), 0.0);
    assert_eq!(
        zero.inverted().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert!(zero.powered(-2).is_err());
    let translation = Transform2::translation([1.0, 2.0]);
    assert_eq!(
        translation.inverted().unwrap().translation_part(),
        Xy::new(-1.0, -2.0)
    );
    assert_eq!(
        translation.powered(5).unwrap().translation_part(),
        Xy::new(5.0, 10.0)
    );
    assert_eq!(translation.powered(0).unwrap(), Transform2::default());
    let scale = Transform2::scaling(Point2::new(1.0, 0.0), 2.0);
    let squared = scale.powered(2).unwrap();
    assert_eq!(squared.scale_factor(), 4.0);
    assert_eq!(squared.translation_part(), Xy::new(-3.0, 0.0));
    assert_eq!(scale.powered(-1).unwrap(), scale.inverted().unwrap());
    let mirror = Transform2::mirror_point(Point2::new(1.0, 0.0));
    assert_eq!(mirror.powered(4).unwrap(), Transform2::default());
    assert_eq!(mirror.powered(-2).unwrap(), Transform2::default());
    assert_eq!(mirror.powered(3).unwrap(), mirror);
    let mut in_place = scale;
    in_place.power(2).unwrap();
    assert_eq!(in_place, squared);
    in_place.invert().unwrap();
    assert_eq!(in_place.scale_factor(), 0.25);
}

#[test]
fn rotation_power_reproduces_the_aliased_native_square() {
    let rotation = Transform2::rotation(Point2::default(), 0.3);
    let cubed = rotation.powered(3).unwrap();
    // OCCT squares the temporary matrix with Tempmatrix.Multiply(Tempmatrix).
    assert_eq!(cubed.value(1, 2).unwrap(), -0.759823872010906);
    assert_eq!(cubed.value(2, 2).unwrap(), 0.5456310371035684);
}

#[test]
fn products_follow_the_native_branch_table() {
    let mirror = Transform2::mirror_point(Point2::new(1.0, 0.0));
    let product = mirror * Transform2::mirror_point(Point2::new(3.0, 0.0));
    assert_eq!(product.form(), TransformForm::Translation);
    assert_eq!(product.scale_factor(), 1.0);
    assert_eq!(product.translation_part(), Xy::new(-4.0, 0.0));
    let axis_mirror = Transform2::mirror_axis(Axis2d::default());
    assert_eq!((axis_mirror * axis_mirror).form(), TransformForm::Rotation);
    let scale = Transform2::scaling(Point2::default(), 2.0);
    let mut composed = Transform2::translation([1.0, 0.0]);
    composed *= scale;
    assert_eq!(composed.form(), TransformForm::Scale);
    let mut pre = Transform2::translation([1.0, 0.0]);
    pre.pre_multiply(scale);
    assert_eq!(pre.form(), TransformForm::Scale);
    assert_eq!(pre.translation_part(), Xy::new(2.0, 0.0));
    let rotation = Transform2::rotation(Point2::new(1.0, 1.0), 0.5);
    let mut left = rotation;
    left.pre_multiply(scale);
    assert_eq!(left, scale.multiplied(rotation));
}

#[test]
fn copy_from_transform3_keeps_form_scale_and_planar_part() {
    let axis3 = Axis1::new(Point3::new(1.0, 2.0, 0.0), Direction3::Z);
    let rotation = Transform3::rotation(axis3, FRAC_PI_2).unwrap();
    let planar = Transform2::from(rotation);
    assert_eq!(planar.form(), TransformForm::Rotation);
    let expected = Transform2::rotation(Point2::new(1.0, 2.0), FRAC_PI_2);
    let image = planar.apply_coordinates(Xy::new(3.0, 2.0));
    let reference = expected.apply_coordinates(Xy::new(3.0, 2.0));
    assert!((image - reference).modulus() < 1e-15);
    let scale = Transform3::scaling(Point3::new(1.0, 1.0, 1.0), 2.0).unwrap();
    let planar = Transform2::from(scale);
    assert_eq!(planar.form(), TransformForm::Scale);
    assert_eq!(planar.translation_part(), Xy::new(-1.0, -1.0));
}

#[test]
fn general_transform_round_trips_similarities() {
    let rotation = Transform2::rotation(Point2::new(1.0, 2.0), 0.3);
    let general = GeneralTransform2::from(rotation);
    assert_eq!(general.form(), TransformForm::Rotation);
    assert_eq!(general.trsf2d().unwrap(), rotation);
    assert_eq!(general.value(1, 3).unwrap(), rotation.value(1, 3).unwrap());
    // A negative power inverts, then gp_Trsf2d::Power receives the negative power again, so
    // T^-2 is (T^-1)^-1 squared: T^2 up to the rounding of the double inversion.
    let (negative, positive) = (general.powered(-2).unwrap(), general.powered(2).unwrap());
    assert_eq!(negative.vectorial_part(), positive.vectorial_part());
    assert!((negative.translation_part() - positive.translation_part()).modulus() < 1e-15);
    assert!(
        (negative.translation_part()
            - general
                .powered(2)
                .unwrap()
                .inverted()
                .unwrap()
                .translation_part())
        .modulus()
            > 1.0
    );
    let mut other = GeneralTransform2::default();
    other.set_trsf2d(rotation);
    assert_eq!(other, general);
    assert_eq!(
        general.multiplied(general).unwrap().trsf2d().unwrap(),
        rotation * rotation
    );
    let mut pre = general;
    pre.pre_multiply(GeneralTransform2::from(Transform2::translation([1.0, 0.0])))
        .unwrap();
    // A translation applied after a rotation keeps the Rotation form (about a moved center).
    assert_eq!(pre.form(), TransformForm::Rotation);
    assert_eq!(
        general.inverted().unwrap().trsf2d().unwrap(),
        rotation.inverted().unwrap()
    );
    assert_eq!(general.powered(0).unwrap(), GeneralTransform2::default());
    assert_eq!(general.powered(1).unwrap(), general);
}

#[test]
fn general_transform_other_form_members() {
    let matrix = Matrix2::new([[2.0, 1.0], [0.0, 1.0]]);
    let mut general = GeneralTransform2::new(matrix, Xy::new(1.0, 0.0));
    assert_eq!(general.form(), TransformForm::Other);
    assert!(!general.is_negative());
    assert!(!general.is_singular());
    assert_eq!(
        general.trsf2d().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(general.transformed(Xy::new(1.0, 1.0)), Xy::new(4.0, 1.0));
    assert_eq!(general.transforms_coordinates(1.0, 1.0), (4.0, 1.0));
    let mut xy = Xy::new(1.0, 1.0);
    general.transforms(&mut xy);
    assert_eq!(xy, Xy::new(4.0, 1.0));
    let squared = general.powered(2).unwrap();
    assert_eq!(squared.vectorial_part(), matrix * matrix);
    assert_eq!(squared.translation_part(), Xy::new(3.0, 0.0));
    let inverse = general.inverted().unwrap();
    assert_eq!(inverse.transformed(Xy::new(4.0, 1.0)), Xy::new(1.0, 1.0));
    general.set_value(2, 3, 5.0).unwrap();
    assert_eq!(general.value(2, 3).unwrap(), 5.0);
    assert!(general.set_value(3, 1, 0.0).is_err());
    assert!(general.value(1, 4).is_err());
    general.set_translation_part(Xy::new(0.0, 0.0));
    assert_eq!(general.form(), TransformForm::Other);
    let mut identity = GeneralTransform2::default();
    identity.set_translation_part(Xy::new(1.0, 0.0));
    assert_eq!(identity.form(), TransformForm::Translation);
    identity.set_vectorial_part(matrix);
    assert_eq!(
        (identity.form(), identity.value(1, 1).unwrap()),
        (TransformForm::Other, 2.0)
    );
}

#[test]
fn general_transform_affinity_fixes_the_axis() {
    let affinity = GeneralTransform2::affinity(axis(0.0, 1.0, 1.0, 0.0), 3.0);
    assert_eq!(affinity.transformed(Xy::new(5.0, 1.0)), Xy::new(5.0, 1.0));
    assert_eq!(affinity.transformed(Xy::new(0.0, 2.0)), Xy::new(0.0, 4.0));
    let mut setter = GeneralTransform2::default();
    setter.set_affinity(axis(0.0, 1.0, 1.0, 0.0), 3.0);
    assert_eq!(setter, affinity);
    let product = (affinity * GeneralTransform2::default()).unwrap();
    assert_eq!(product.form(), TransformForm::Other);
}
