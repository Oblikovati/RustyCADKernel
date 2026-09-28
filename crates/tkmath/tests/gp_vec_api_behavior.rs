//! Supplemental gp_Vec API: coordinate access, operators, linear forms, symmetries,
//! transformations and the exception paths the upstream suite does not reach.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Direction3, Point3, Transform3, Vector3, Xyz, vector_with_null_magnitude,
};

#[test]
fn coordinates_are_read_and_replaced_by_index_and_name() {
    let mut vector = Vector3::default();
    vector.set_coordinates(1.0, 2.0, 3.0);
    assert_eq!(vector.coord(3).unwrap(), 3.0);
    assert_eq!(
        vector.coord(4).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(vector.set_coord(0, 9.0).is_err());
    vector.set_coord(2, 5.0).unwrap();
    vector.set_x(-1.0);
    vector.set_z(7.0);
    assert_eq!(vector, Vector3::new(-1.0, 5.0, 7.0));
    vector.set_y(0.5);
    vector.set_xyz(Xyz::new(4.0, 5.0, 6.0));
    assert_eq!(*vector.xyz(), Xyz::new(4.0, 5.0, 6.0));
}

#[test]
fn operators_follow_the_named_members() {
    let (a, b) = (Vector3::new(1.0, 2.0, 3.0), Vector3::new(-2.0, 0.5, 4.0));
    let mut c = a;
    c += b;
    assert_eq!(c, a + b);
    c -= b;
    assert_eq!(c, a);
    c *= 2.0;
    assert_eq!(c, 2.0 * a);
    c /= 2.0;
    assert_eq!(c, a);
    assert_eq!(a ^ b, a.crossed(b));
    c ^= b;
    assert_eq!(c, a.crossed(b));
    assert_eq!(a * b, a.dot(b));
    assert_eq!(-a, a.reversed());
    assert_eq!(
        a / 0.0,
        Vector3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY)
    );
}

#[test]
fn cross_products_keep_the_nested_order() {
    let (a, b, c) = (
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(2.0, 0.0, 1.0),
    );
    assert_eq!(a.cross_crossed(b, c), a.crossed(b.crossed(c)));
    let mut d = a;
    d.cross_cross(b, c);
    assert_eq!(d, a.cross_crossed(b, c));
    assert_eq!(a.cross_square_magnitude(b), a.crossed(b).square_magnitude());
    let mut e = a;
    e.cross(b);
    assert_eq!(e, Vector3::new(-3.0, 0.0, 1.0));
}

#[test]
fn every_linear_form_matches_its_expression() {
    let (a, b, c, d) = (
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(-1.0, 0.5, 2.0),
        Vector3::new(0.25, 4.0, -3.0),
        Vector3::new(9.0, 8.0, 7.0),
    );
    let mut v = Vector3::default();
    v.set_linear_form_three_offset([2.0, 3.0, 4.0], [a, b, c], d);
    assert_eq!(v, a * 2.0 + b * 3.0 + c * 4.0 + d);
    v.set_linear_form_three([2.0, 3.0, 4.0], [a, b, c]);
    assert_eq!(v, a * 2.0 + b * 3.0 + c * 4.0);
    v.set_linear_form_offset(2.0, a, 3.0, b, c);
    assert_eq!(v, a * 2.0 + b * 3.0 + c);
    v.set_linear_form(2.0, a, 3.0, b);
    assert_eq!(v, a * 2.0 + b * 3.0);
    v.set_scaled_sum(2.0, a, b);
    assert_eq!(v, a * 2.0 + b);
    v.set_sum(a, b);
    assert_eq!(v, a + b);
}

#[test]
fn is_equal_checks_the_angle_only_above_the_linear_tolerance() {
    let a = Vector3::new(1.0, 0.0, 0.0);
    assert!(
        a.is_equal(Vector3::new(1.0, 1e-9, 0.0), 1e-7, 1e-6)
            .unwrap()
    );
    assert!(!a.is_equal(Vector3::new(0.0, 1.0, 0.0), 1e-7, 1e-6).unwrap());
    assert!(a.is_equal(Vector3::default(), 2.0, 0.0).unwrap());
    // With a negative linear tolerance the length test fails first and `&&` skips Angle,
    // so null vectors do not raise.
    let zero = Vector3::default();
    assert!(!zero.is_equal(zero, -1.0, 0.0).unwrap());
    let failure = a.angle(zero).unwrap_err();
    assert_eq!(failure.exception_type(), "gp_VectorWithNullMagnitude");
    assert!(
        failure.message().contains("[1.0, 0.0]"),
        "{}",
        failure.message()
    );
    let reference = a.angle_with_ref(a, zero).unwrap_err();
    assert_eq!(reference.exception_type(), "gp_VectorWithNullMagnitude");
    assert_eq!(
        vector_with_null_magnitude("m").exception_type(),
        "gp_VectorWithNullMagnitude"
    );
}

#[test]
fn normalization_reports_the_offending_vector() {
    let failure = Vector3::default().normalized().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert!(
        failure.message().contains("(0, 0, 0)"),
        "{}",
        failure.message()
    );
    let mut unit = Vector3::new(0.0, 3.0, 4.0);
    unit.normalize().unwrap();
    assert_eq!(unit, Vector3::new(0.0, 0.6, 0.8));
}

#[test]
fn mirrors_reflect_about_vectors_axes_and_planes() {
    let v = Vector3::new(1.0, 2.0, 3.0);
    assert_eq!(
        v.mirrored_vector(Vector3::new(0.0, 0.0, 2.0)),
        Vector3::new(-1.0, -2.0, 3.0)
    );
    // A null or NaN mirror vector leaves the vector unchanged, as the native magnitude test.
    assert_eq!(v.mirrored_vector(Vector3::default()), v);
    assert_eq!(v.mirrored_vector(Vector3::new(f64::NAN, 0.0, 0.0)), v);
    let axis = Axis1::new(Point3::new(5.0, 5.0, 5.0), Direction3::X);
    assert_eq!(v.mirrored_axis(axis), Vector3::new(1.0, -2.0, -3.0));
    let plane = Axis2::default();
    // The plane mirror reflects about the normalized `N ^ (N ^ V)`, so it rounds.
    let reflected = v.mirrored_plane(plane);
    assert!((reflected - Vector3::new(1.0, 2.0, -3.0)).magnitude() <= 1e-15);
    // A vector along the plane normal takes the reversal branch.
    assert_eq!(
        Vector3::new(0.0, 0.0, 2.0).mirrored_plane(plane),
        Vector3::new(-0.0, -0.0, -2.0)
    );
    let mut w = v;
    w.mirror_vector(Vector3::new(1.0, 0.0, 0.0));
    w.mirror_axis(axis);
    w.mirror_plane(plane);
    assert_eq!(
        w,
        v.mirrored_vector(Vector3::new(1.0, 0.0, 0.0))
            .mirrored_axis(axis)
            .mirrored_plane(plane)
    );
}

#[test]
fn motions_follow_the_transformation_form() {
    let v = Vector3::new(1.0, 2.0, 3.0);
    let quarter = v
        .rotated(Axis1::default(), std::f64::consts::FRAC_PI_2)
        .unwrap();
    assert!((quarter.x() + 2.0).abs() <= 1e-15 && (quarter.y() - 1.0).abs() <= 1e-15);
    let mut rotated = v;
    rotated
        .rotate(Axis1::default(), std::f64::consts::FRAC_PI_2)
        .unwrap();
    assert_eq!(rotated, quarter);
    assert_eq!(v.scaled(-2.0), Vector3::new(-2.0, -4.0, -6.0));
    let translation = Transform3::translation_vector(Vector3::new(9.0, 9.0, 9.0));
    assert_eq!(v.transformed(translation), v);
    assert_eq!(
        v.transformed(Transform3::mirror_point(Point3::default())),
        -v
    );
    let scale = Transform3::scaling(Point3::new(1.0, 1.0, 1.0), 3.0).unwrap();
    assert_eq!(v.transformed(scale), v * 3.0);
    let mut moved = v;
    moved.scale(2.0);
    moved.transform(Transform3::rotation(Axis1::default(), 0.0).unwrap());
    assert_eq!(moved, v * 2.0);
}

#[test]
fn dump_json_writes_the_native_fragment() {
    let mut stream = DumpStream::default();
    Vector3::new(1.0, 2.5, -3.0).dump_json(&mut stream).unwrap();
    assert_eq!(stream.text(), "\"gp_Vec\": [1, 2.5, -3]");
}
