//! Supplemental gp_Trsf and gp_GTrsf behavior pinned against native OCCT.
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis3, Direction2, Direction3, GeneralTransform3, Matrix3, Point2,
    Point3, Quaternion, Transform2, Transform3, TransformForm, Vector3, Xyz,
};

fn z_axis(x: f64) -> Axis1 {
    Axis1::new(Point3::new(x, 0.0, 0.0), Direction3::Z)
}

#[test]
fn mirrors_store_native_forms() {
    let point = Transform3::mirror_point(Point3::new(1.0, 2.0, 3.0));
    assert_eq!(
        (point.form(), point.scale_factor()),
        (TransformForm::PointMirror, -1.0)
    );
    assert!(point.is_negative(), "gp_Trsf tests the scale sign");
    assert_eq!(point.apply(Point3::default()), Point3::new(2.0, 4.0, 6.0));
    let axis = Transform3::mirror_axis(z_axis(1.0));
    assert_eq!(
        (axis.form(), axis.scale_factor()),
        (TransformForm::AxisMirror, 1.0)
    );
    assert_eq!(
        axis.apply(Point3::new(0.0, 0.0, 5.0)),
        Point3::new(2.0, 0.0, 5.0)
    );
    let plane = Transform3::mirror_plane(Axis2::new(Point3::default(), Direction3::Z).unwrap());
    assert_eq!(
        (plane.form(), plane.scale_factor()),
        (TransformForm::PlaneMirror, -1.0)
    );
    assert_eq!(
        plane.apply(Point3::new(1.0, 2.0, 3.0)),
        Point3::new(1.0, 2.0, -3.0)
    );
    let mut setter = Transform3::default();
    setter.set_mirror_plane(Axis2::new(Point3::default(), Direction3::Z).unwrap());
    assert_eq!(setter, plane);
    setter.set_mirror_axis(z_axis(1.0));
    assert_eq!(setter, axis);
    setter.set_mirror_point(Point3::new(1.0, 2.0, 3.0));
    assert_eq!(setter, point);
    // Even powers of a mirror are the identity; odd powers keep it.
    assert_eq!(axis.powered(4).unwrap(), Transform3::default());
    assert_eq!(
        point.powered(-3).unwrap().form(),
        TransformForm::PointMirror
    );
}

#[test]
fn mirror_products_follow_the_branch_table() {
    let a = Transform3::mirror_point(Point3::new(1.0, 0.0, 0.0));
    let b = Transform3::mirror_point(Point3::new(3.0, 0.0, 0.0));
    let product = a * b;
    assert_eq!(
        (product.form(), product.translation_part()),
        (TransformForm::Translation, Xyz::new(-4.0, 0.0, 0.0))
    );
    let axis = Transform3::mirror_axis(z_axis(0.0));
    let mut twice = axis;
    twice *= axis;
    assert_eq!(twice.form(), TransformForm::Rotation);
    let mut pre = axis;
    pre.pre_multiply(axis);
    assert_eq!(pre, twice);
}

#[test]
fn scale_factor_translation_and_rotation_part_rederive_forms() {
    let mut t = Transform3::default();
    t.set_scale_factor(-1.0).unwrap();
    assert_eq!(t.form(), TransformForm::PointMirror);
    t.set_scale_factor(3.0).unwrap();
    assert_eq!(t.form(), TransformForm::Scale);
    t.set_scale_factor(1.0).unwrap();
    assert_eq!(t.form(), TransformForm::Identity);
    assert!(t.set_scale_factor(0.0).is_err());
    t.set_translation_part(Vector3::new(1e-170, 0.0, 0.0));
    assert_eq!(
        t.form(),
        TransformForm::Identity,
        "a squared length below Resolution is null"
    );
    t.set_translation_part(Vector3::new(1.0, 0.0, 0.0));
    assert_eq!(t.form(), TransformForm::Translation);
    t.set_rotation_part(Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), 0.5).unwrap());
    assert_eq!(t.form(), TransformForm::Compound);
    let mut r = Transform3::rotation(z_axis(0.0), 0.5).unwrap();
    r.set_rotation_part(Quaternion::default());
    assert_eq!(r.form(), TransformForm::Identity);
    assert_eq!(r.matrix(), Transform3::default().matrix());
    r.set_form(TransformForm::Other);
    assert_eq!(r.form(), TransformForm::Other);
}

#[test]
fn set_values_factors_a_cube_root_scale() {
    let mut t = Transform3::default();
    t.set_values([
        [0.0, -2.0, 0.0, 1.0],
        [2.0, 0.0, 0.0, 2.0],
        [0.0, 0.0, 2.0, 3.0],
    ])
    .unwrap();
    assert_eq!(t.scale_factor(), 2.0);
    assert_eq!(t.form(), TransformForm::Compound);
    assert_eq!(t.translation_part(), Xyz::new(1.0, 2.0, 3.0));
    assert_eq!(t.value(1, 2).unwrap(), -2.0);
    let error = t
        .set_values([
            [1.0, 2.0, 0.0, 0.0],
            [2.0, 4.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ])
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(
        t.vectorial_part(),
        t.matrix() * 2.0,
        "compound forms scale the whole matrix"
    );
}

#[test]
fn frame_changes_quaternions_and_planar_copies() {
    let from = Axis3::from_normal(Point3::new(1.0, 0.0, 0.0), Direction3::Y).unwrap();
    let to = Axis3::default();
    let change = Transform3::transformation_between(from, to);
    assert_eq!(change.form(), TransformForm::Compound);
    let mut setter = Transform3::default();
    setter.set_transformation_between(from, to);
    assert_eq!(setter, change);
    setter.set_displacement(from, to);
    assert_eq!(setter, Transform3::displacement(from, to));
    let q = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), 0.3).unwrap();
    let moved = Transform3::rotation_translation(q, Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(moved.translation_part(), Xyz::new(1.0, 2.0, 3.0));
    let (axis, angle) = moved.rotation_axis_angle();
    assert!((angle - 0.3).abs() < 1e-15 && (axis.z() - 1.0).abs() < 1e-15);
    setter.set_rotation_quaternion(q);
    assert_eq!(setter.form(), TransformForm::Rotation);
    let planar = Transform2::mirror_axis(Axis2d::new(Point2::default(), Direction2::X));
    let spatial = Transform3::from(planar);
    // A planar axis mirror becomes the half-turn about that axis: scale 1, negated matrix.
    assert_eq!(
        (spatial.form(), spatial.scale_factor()),
        (TransformForm::AxisMirror, 1.0)
    );
    assert!(spatial.matrix().coefficients()[0][2].is_sign_negative());
    assert_eq!(
        spatial.apply(Point3::new(1.0, 2.0, 3.0)),
        Point3::new(1.0, -2.0, -3.0)
    );
    assert_eq!(
        spatial.transforms_coordinates(0.0, 1.0, 0.0),
        (0.0, -1.0, 0.0)
    );
}

#[test]
fn mat4_and_json_round_trip() {
    let t = Transform3::rotation(z_axis(1.0), 0.5).unwrap();
    let mut double = Mat4::<f64>::zero();
    t.get_mat4(&mut double);
    assert_eq!(double.get_value(0, 3), t.value(1, 4).unwrap());
    assert_eq!(double.get_row(3).data(), [0.0, 0.0, 0.0, 1.0]);
    let mut single = Mat4::<f32>::zero();
    Transform3::default().get_mat4(&mut single);
    assert!(single.is_identity());
    let mut dump = DumpStream::default();
    Transform3::translation([1.0, 2.0, 3.0])
        .dump_json(&mut dump)
        .unwrap();
    assert_eq!(
        dump.text(),
        "\"Location\": [1, 2, 3], \"Matrix\": [1, 0, 0, 0, 1, 0, 0, 0, 1], \"shape\": 2, \"scale\": 1"
    );
    let mut parsed = Transform3::default();
    let mut position = 1;
    assert!(parsed.init_from_json(dump.text(), &mut position).unwrap());
    assert_eq!(parsed, Transform3::translation([1.0, 2.0, 3.0]));
    let text = dump.text().replace("\"shape\": 2", "\"shape\": 9");
    let mut invalid = Transform3::default();
    let mut position = 1;
    assert!(invalid.init_from_json(&text, &mut position).is_err());
}

#[test]
fn general_transform_members() {
    let rotation = Transform3::rotation(z_axis(1.0), 0.5).unwrap();
    let mut general = GeneralTransform3::from(rotation);
    assert_eq!(general.trsf().unwrap(), rotation);
    // A negative general power ignores the sign; a similarity power inverts.
    let matrix = Matrix3::new([[2.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    let skew = GeneralTransform3::new(matrix, Xyz::new(1.0, 0.0, 0.0));
    assert_eq!(skew.powered(-2).unwrap(), skew.powered(2).unwrap());
    assert_eq!(
        general.powered(-2).unwrap().trsf().unwrap(),
        rotation.powered(-2).unwrap()
    );
    assert_eq!((skew * skew).vectorial_part(), matrix * matrix);
    let mut pre = skew;
    pre.pre_multiply(GeneralTransform3::from(Transform3::translation([
        0.0, 1.0, 0.0,
    ])));
    assert_eq!(pre.translation_part(), Xyz::new(1.0, 1.0, 0.0));
    general.set_value(1, 4, 7.0).unwrap();
    assert_eq!(
        general.form(),
        TransformForm::Rotation,
        "column 4 keeps the form"
    );
    let mut scaled = GeneralTransform3::from(Transform3::scaling(Point3::default(), 2.0).unwrap());
    scaled.set_value(1, 2, 0.5).unwrap();
    // The scale is folded into the matrix before the cell is written.
    assert_eq!(
        (scaled.form(), scaled.value(1, 1).unwrap()),
        (TransformForm::Other, 2.0)
    );
    assert!(scaled.set_value(0, 1, 0.0).is_err());
    let mut formed = GeneralTransform3::new(matrix, Xyz::default());
    formed.set_form().unwrap();
    assert_eq!(formed.form(), TransformForm::Other);
    let mut orthogonal = GeneralTransform3::new(rotation.matrix() * 3.0, Xyz::default());
    orthogonal.set_form().unwrap();
    assert_eq!(orthogonal.form(), TransformForm::Compound);
    assert!(
        GeneralTransform3::new(Matrix3::default(), Xyz::default())
            .set_form()
            .is_err()
    );
    let mut m4 = Mat4::<f64>::identity();
    m4.set_value(0, 3, 4.0);
    formed.set_mat4(&m4);
    assert_eq!(
        (formed.form(), formed.translation_part()),
        (TransformForm::Other, Xyz::new(4.0, 0.0, 0.0))
    );
    let mut out = Mat4::<f64>::zero();
    formed.get_mat4(&mut out);
    assert_eq!(out, m4);
    assert_eq!(
        formed.transforms_coordinates(1.0, 1.0, 1.0),
        (5.0, 1.0, 1.0)
    );
    let mut dump = DumpStream::default();
    formed.dump_json(&mut dump, 0).unwrap();
    assert_eq!(
        dump.text(),
        "\"className\": \"gp_GTrsf\", \"shape\": 8, \"scale\": 0"
    );
}
