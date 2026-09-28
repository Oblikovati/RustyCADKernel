//! Supplemental gp_Vec2d, gp_Pnt2d and gp_Lin2d behavior pinned against native OCCT.
use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Point2, Transform2, Vector2, Xy};

#[test]
fn vector2_accessors_operators_and_normalization() {
    let mut vector = Vector2::new(3.0, 4.0);
    assert_eq!(vector.coord(1).unwrap(), 3.0);
    assert!(vector.set_coord(3, 1.0).is_err());
    vector.set_x(6.0);
    vector.set_y(8.0);
    assert_eq!(vector.magnitude(), 10.0);
    vector.set_coordinates(3.0, 4.0);
    vector.set_xy(Xy::new(3.0, 4.0));
    let other = Vector2::new(1.0, 2.0);
    assert_eq!(vector * other, 11.0);
    assert_eq!(vector ^ other, 2.0);
    assert_eq!(vector.cross_magnitude(Vector2::new(-1.0, 0.0)), 4.0);
    assert_eq!(vector.cross_square_magnitude(other), 4.0);
    assert_eq!(-vector, vector.reversed());
    assert_eq!(2.0 * vector, vector * 2.0);
    let mut sum = vector;
    sum += other;
    sum -= other;
    sum *= 2.0;
    sum /= 2.0;
    assert_eq!(sum, vector);
    vector.normalize().unwrap();
    assert_eq!(vector, Vector2::new(0.6, 0.8));
    let mut zero = Vector2::default();
    assert_eq!(
        zero.normalize().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    vector.reverse();
    assert_eq!(vector, Vector2::new(-0.6, -0.8));
}

#[test]
fn vector2_normal_keeps_native_multiplication() {
    let normal = Vector2::new(1.0, 2.0).normal();
    assert_eq!(normal, Vector2::new(2.0, -1.0));
    // (-1)*X is a multiplication, so a zero X becomes -0.0.
    assert!(Vector2::new(0.0, 1.0).normal().y().is_sign_negative());
}

#[test]
fn vector2_is_equal_checks_angles_only_for_long_vectors() {
    let (a, b) = (Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0));
    assert!(!a.is_equal(b, 1e-7, 1e-12).unwrap());
    assert!(
        a.is_equal(b, 2.0, 1e-12).unwrap(),
        "both shorter than the tolerance"
    );
    let zero = Vector2::default();
    assert!(!a.is_equal(zero, 1e-7, 0.1).unwrap());
    // A negative linear tolerance reaches the angle of a zero vector, which raises.
    assert_eq!(
        zero.is_equal(zero, -1.0, 0.1).unwrap_err().exception_type(),
        "gp_VectorWithNullMagnitude"
    );
}

#[test]
fn vector2_geometry_members() {
    let vector = Vector2::new(1.0, 0.0);
    let mirrored = vector.mirrored_vector(Vector2::new(0.0, 2.0));
    assert_eq!(mirrored, Vector2::new(-1.0, 0.0));
    // A null mirror vector leaves the vector unchanged.
    assert_eq!(vector.mirrored_vector(Vector2::default()), vector);
    let mut copy = vector;
    copy.mirror_vector(Vector2::new(0.0, 2.0));
    assert_eq!(copy, mirrored);
    copy = vector;
    copy.mirror_axis(Axis2d::new(Point2::default(), Direction2::Y));
    assert_eq!(copy, Vector2::new(-1.0, 0.0));
    copy = vector;
    copy.rotate(std::f64::consts::PI);
    assert_eq!(copy, vector.rotated(std::f64::consts::PI));
    copy.scale(3.0);
    assert_eq!(copy, vector.rotated(std::f64::consts::PI).scaled(3.0));
    copy = vector;
    copy.transform(Transform2::mirror_point(Point2::new(5.0, 5.0)));
    assert_eq!(copy, Vector2::new(-1.0, -0.0));
    let scaled = vector.transformed(Transform2::scaling(Point2::new(5.0, 5.0), 3.0));
    assert_eq!(scaled, Vector2::new(3.0, 0.0));
    let mut form = Vector2::default();
    form.set_linear_form_offset(2.0, vector, 3.0, Vector2::new(0.0, 1.0), vector);
    assert_eq!(form, Vector2::new(3.0, 3.0));
    form.set_scaled_sum(2.0, vector, vector);
    assert_eq!(form, Vector2::new(3.0, 0.0));
    form.set_sum(vector, vector);
    assert_eq!(form, Vector2::new(2.0, 0.0));
}

#[test]
fn point2_members_and_transform_shortcuts() {
    let mut point = Point2::new(1.0, 2.0);
    assert_eq!(point.coord(2).unwrap(), 2.0);
    assert!(point.set_coord(0, 1.0).is_err());
    point.set_xy(Xy::new(2.0, 2.0));
    point.change_coord().set_x(3.0);
    assert_eq!(point, Point2::new(3.0, 2.0));
    point.translate(Vector2::new(1.0, 1.0));
    point.translate_between(Point2::new(1.0, 1.0), Point2::new(0.0, 0.0));
    assert_eq!(point, Point2::new(3.0, 2.0));
    point.scale(Point2::default(), 2.0);
    point.mirror_point(Point2::default());
    assert_eq!(point, Point2::new(-6.0, -4.0));
    point.mirror_axis(Axis2d::default());
    assert_eq!(point, Point2::new(-6.0, 4.0));
    point.rotate(Point2::default(), 0.0);
    point.transform(Transform2::scaling(Point2::new(1.0, 1.0), 2.0));
    assert_eq!(point, Point2::new(-13.0, 7.0));
    point.transform(Transform2::mirror_point(Point2::default()));
    assert_eq!(point, Point2::new(13.0, -7.0));
}

#[test]
fn line2_members() {
    let mut line = Line2::new(Point2::new(1.0, 0.0), Direction2::Y);
    assert_eq!(line.coefficients(), (1.0, -0.0, -1.0));
    assert_eq!(line.distance(Point2::new(4.0, 7.0)), 3.0);
    assert_eq!(line.square_distance(Point2::new(4.0, 7.0)), 9.0);
    assert!(line.contains(Point2::new(1.0, 9.0), 0.0));
    let parallel = Line2::new(Point2::new(-2.0, 0.0), Direction2::NEG_Y);
    assert_eq!(line.distance_line(parallel), 3.0);
    assert_eq!(line.square_distance_line(parallel), 9.0);
    let crossing = Line2::new(Point2::default(), Direction2::X);
    assert_eq!(line.distance_line(crossing), 0.0);
    assert_eq!(line.angle(crossing), -std::f64::consts::FRAC_PI_2);
    let normal = line.normal(Point2::new(0.0, 5.0)).unwrap();
    assert_eq!(normal.direction(), Direction2::NEG_X);
    line.set_direction(Direction2::X);
    line.set_location(Point2::new(0.0, 1.0));
    assert_eq!(
        line.position(),
        Axis2d::new(Point2::new(0.0, 1.0), Direction2::X)
    );
    line.set_position(Axis2d::default());
    line.reverse();
    assert_eq!(line, Line2::from(Axis2d::default()).reversed());
    assert!(Line2::from_equation(0.0, 0.0, 1.0).is_err());
}

#[test]
fn line2_translation_overloads_differ_in_order() {
    let line = Line2::new(Point2::new(0.3, 0.0), Direction2::X);
    let (from, to) = (Point2::new(0.6, 0.0), Point2::new(-0.8, 0.0));
    // The copy adds gp_Vec2d(from,to); the in-place member adds `to` and then subtracts `from`.
    let copied = line.translated_between(from, to);
    let mut in_place = line;
    in_place.translate_between(from, to);
    assert_eq!(copied.location().x(), -1.0999999999999999);
    assert_eq!(in_place.location().x(), -1.1);
    assert_ne!(copied.location(), in_place.location());
    let mut moved = line;
    moved.translate(Vector2::new(1.0, 0.0));
    moved.rotate(Point2::default(), 0.0);
    moved.scale(Point2::default(), -1.0);
    assert_eq!(moved.direction(), Direction2::NEG_X);
    moved.mirror_point(Point2::default());
    moved.mirror_axis(Axis2d::default());
    moved.transform(Transform2::default());
    assert_eq!(moved.location(), Point2::new(1.3, -0.0));
}
