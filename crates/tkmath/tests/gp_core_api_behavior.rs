//! Supplemental gp_Dir, gp_Pnt, gp_Lin and gp_Pln members the upstream suites do not reach:
//! renormalizing setters, the writable point view, the std::hash/std::equal_to
//! specializations, and every line/plane overload.
use rustycad_tkernel::ncollection::{HashKey, HashPolicy};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Direction3, Line3, Plane3, Point3, PointHashPolicy, Transform3, Vector3,
    Xyz,
};

#[test]
fn direction_setters_renormalize_and_keep_the_value_on_failure() {
    let mut direction = Direction3::default();
    direction.set_coordinates(0.0, 0.0, 2.0).unwrap();
    assert_eq!(direction, Direction3::Z);
    direction.set_x(1.0).unwrap();
    assert_eq!(direction.xyz(), Xyz::new(1.0, 0.0, 1.0) / 2.0_f64.sqrt());
    let before = direction;
    let failure = direction.set_coordinates(0.0, 0.0, 0.0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert!(
        failure.message().contains("(0, 0, 0)"),
        "{}",
        failure.message()
    );
    assert_eq!(direction, before);
    // The index is checked before the norm, as the native raise order.
    let out_of_range = direction.set_coord(4, 0.0).unwrap_err();
    assert_eq!(out_of_range.exception_type(), "Standard_OutOfRange");
    direction.set_coord(2, 3.0).unwrap();
    direction.set_y(0.0).unwrap();
    direction.set_z(0.0).unwrap();
    assert_eq!(direction, Direction3::X);
    direction.set_xyz(Xyz::new(0.0, -5.0, 0.0)).unwrap();
    assert_eq!(direction.coord(2).unwrap(), -1.0);
    assert!(direction.coord(0).is_err());
    assert_eq!(-direction, direction.reversed());
    assert_eq!(direction * Direction3::Y, -1.0);
}

#[test]
fn change_coord_writes_the_point_back() {
    let mut point = Point3::new(1.0, 2.0, 3.0);
    point.change_coord().set_x(-4.0);
    *point.change_coord() += Xyz::new(1.0, 1.0, 1.0);
    assert_eq!(point, Point3::new(-3.0, 3.0, 4.0));
    assert_eq!(point.change_coord().z(), 4.0);
}

#[test]
fn point_hash_policy_matches_equal_to_within_one_ulp() {
    let policy = PointHashPolicy;
    let point = Point3::new(1.0, -2.5, 1e10);
    let nudged = Point3::new(1.0_f64.next_up(), -2.5, 1e10);
    assert!(policy.equal(&point, &nudged));
    assert!(!policy.equal(&point, &Point3::new(1.0 + 4.0 * f64::EPSILON, -2.5, 1e10)));
    // NaN differences are not "greater", so they compare equal as natively.
    assert!(policy.equal(&Point3::new(f64::NAN, 0.0, 0.0), &Point3::default()));
    assert_eq!(policy.hash(&point), point.occt_hash());
    assert_eq!(point.occt_hash(), Point3::new(1.0, -2.5, 1e10).occt_hash());
    // The origin sums to zero, which libstdc++ hashes to zero.
    assert_eq!(Point3::default().occt_hash(), 0);
}

#[test]
fn line_accessors_and_setters_edit_the_axis() {
    let mut line = Line3::default();
    assert_eq!(line.position(), Axis1::default());
    line.set_location(Point3::new(1.0, 0.0, 0.0));
    line.set_direction(Direction3::X);
    assert_eq!(line, Line3::new(Point3::new(1.0, 0.0, 0.0), Direction3::X));
    line.set_position(Axis1::default());
    assert_eq!(line, Line3::from(Axis1::default()));
    let reversed = line.reversed();
    line.reverse();
    assert_eq!(line, reversed);
    assert_eq!(line.direction(), -Direction3::Z);
}

#[test]
fn line_distances_square_the_line_overload() {
    let line = Line3::default();
    let parallel = Line3::new(Point3::new(3.0, 4.0, 0.0), Direction3::Z);
    assert_eq!(line.square_distance_line(parallel).unwrap(), 25.0);
    let skew = Line3::new(Point3::new(0.0, 0.0, 2.0), Direction3::X);
    assert_eq!(line.distance_line(skew).unwrap(), 0.0);
}

#[test]
fn line_translation_between_points_keeps_both_native_roundings() {
    let line = Line3::new(Point3::new(0.7, 0.0, 0.0), Direction3::Z);
    let (from, to) = (Point3::new(0.1, 0.0, 0.0), Point3::new(0.3, 0.0, 0.0));
    // Translated(P1,P2) adds gp_Vec(P1,P2); Translate(P1,P2) adds P2 then subtracts P1.
    assert_eq!(
        line.translated_between(from, to).location().x(),
        0.8999999999999999
    );
    let mut moved = line;
    moved.translate_between(from, to);
    assert_eq!(moved.location().x(), 0.9);
}

#[test]
fn line_motions_match_their_copying_forms() {
    let line = Line3::new(Point3::new(1.0, 2.0, 3.0), Direction3::X);
    let pivot = Axis1::new(Point3::new(0.0, 1.0, 0.0), Direction3::Y);
    let plane = Axis2::default();
    let mut edited = line;
    edited.mirror_point(Point3::default());
    edited.mirror_axis(pivot);
    edited.mirror_plane(plane);
    edited.rotate(pivot, 0.5).unwrap();
    edited.scale(Point3::new(1.0, 1.0, 1.0), -2.0);
    edited.transform(Transform3::mirror_point(Point3::default()));
    edited.translate(Vector3::new(1.0, 0.0, 0.0));
    let copied = line
        .mirrored_point(Point3::default())
        .mirrored_axis(pivot)
        .mirrored_plane(plane)
        .rotated(pivot, 0.5)
        .unwrap()
        .scaled(Point3::new(1.0, 1.0, 1.0), -2.0)
        .transformed(Transform3::mirror_point(Point3::default()))
        .translated(Vector3::new(1.0, 0.0, 0.0));
    assert_eq!(edited, copied);
}

#[test]
fn plane_equation_constructor_places_the_origin_on_the_branch_axis() {
    let plane = Plane3::from_equation(1.0, 2.0, 0.5, -4.0).unwrap();
    assert_eq!(plane.location(), Point3::new(0.0, 2.0, 0.0));
    let plane = Plane3::from_equation(0.25, 1.0, 3.0, 6.0).unwrap();
    assert_eq!(plane.location(), Point3::new(0.0, 0.0, -2.0));
    let failure = Plane3::from_equation(0.0, 0.0, 0.0, 1.0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}

#[test]
fn plane_accessors_setters_and_orientation() {
    let mut plane = Plane3::default();
    assert_eq!(plane.x_axis(), Axis1::new(Point3::default(), Direction3::X));
    assert_eq!(plane.y_axis(), Axis1::new(Point3::default(), Direction3::Y));
    assert!(plane.direct().unwrap());
    plane.u_reverse();
    assert!(!plane.direct().unwrap());
    plane.v_reverse();
    assert!(plane.direct().unwrap());
    plane.set_location(Point3::new(0.0, 0.0, 1.0));
    plane
        .set_axis(Axis1::new(Point3::new(0.0, 0.0, 2.0), Direction3::X))
        .unwrap();
    assert_eq!(
        plane.axis(),
        Axis1::new(Point3::new(0.0, 0.0, 2.0), Direction3::X)
    );
    plane.set_position(Axis3::default());
    assert_eq!(plane.location(), Point3::default());
    assert_eq!(
        Plane3::from(Axis3::default()).position().direction(),
        Direction3::Z
    );
}

#[test]
fn plane_line_and_plane_distances_are_zero_unless_parallel() {
    let plane = Plane3::default();
    let parallel_line = Line3::new(Point3::new(0.0, 0.0, -3.0), Direction3::X);
    assert_eq!(plane.signed_distance_line(parallel_line), -3.0);
    assert_eq!(plane.distance_line(parallel_line), 3.0);
    assert_eq!(plane.square_distance_line(parallel_line), 9.0);
    assert_eq!(plane.distance_line(Line3::default()), 0.0);
    let parallel = Plane3::new(Point3::new(0.0, 0.0, 2.0), -Direction3::Z).unwrap();
    assert_eq!(plane.signed_distance_plane(parallel), 2.0);
    assert_eq!(plane.square_distance_plane(parallel), 4.0);
    let crossing = Plane3::new(Point3::new(0.0, 0.0, 2.0), Direction3::X).unwrap();
    assert_eq!(plane.distance_plane(crossing), 0.0);
    assert_eq!(plane.square_distance(Point3::new(1.0, 1.0, 3.0)), 9.0);
    assert!(plane.contains_line(
        parallel_line.translated(Vector3::new(0.0, 0.0, 3.0)),
        1e-7,
        1e-12
    ));
    assert!(!plane.contains_line(Line3::default(), 1e-7, 1e-12));
}

#[test]
fn plane_motions_match_their_copying_forms() {
    let plane = Plane3::new(Point3::new(1.0, 2.0, 3.0), Direction3::Y).unwrap();
    let pivot = Axis1::new(Point3::new(0.0, 1.0, 0.0), Direction3::X);
    let mut edited = plane;
    edited.mirror_point(Point3::default());
    edited.mirror_axis(pivot);
    edited.mirror_plane(Axis2::default());
    edited.rotate(pivot, 0.5).unwrap();
    edited.scale(Point3::default(), -1.5);
    edited.transform(Transform3::mirror_axis(pivot));
    edited.translate(Vector3::new(0.0, 1.0, 0.0));
    edited.translate_between(Point3::default(), Point3::new(0.1, 0.2, 0.3));
    let copied = plane
        .mirrored_point(Point3::default())
        .mirrored_axis(pivot)
        .mirrored_plane(Axis2::default())
        .rotated(pivot, 0.5)
        .unwrap()
        .scaled(Point3::default(), -1.5)
        .transformed(Transform3::mirror_axis(pivot))
        .translated(Vector3::new(0.0, 1.0, 0.0))
        .translated_between(Point3::default(), Point3::new(0.1, 0.2, 0.3));
    assert_eq!(
        edited.coefficients().unwrap(),
        copied.coefficients().unwrap()
    );
    assert_eq!(
        edited.position().x_direction(),
        copied.position().x_direction()
    );
}

#[test]
fn plane_dump_json_nests_the_position_unless_depth_is_zero() {
    let mut stream = DumpStream::default();
    Plane3::default().dump_json(&mut stream, -1).unwrap();
    assert_eq!(
        stream.text(),
        "\"Position\": {\"Location\": [0, 0, 0], \"Direction\": [0, 0, 1], \
         \"XDirection\": [1, 0, 0], \"YDirection\": [0, 1, 0]}"
    );
    let mut empty = DumpStream::default();
    Plane3::default().dump_json(&mut empty, 0).unwrap();
    assert_eq!(empty.text(), "");
}
