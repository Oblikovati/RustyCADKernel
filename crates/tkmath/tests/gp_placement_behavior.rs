//! Supplemental checks for the gp_Ax1/Ax2/Ax3 members added for the conics and quadrics.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, Transform3, Vector3};
use std::f64::consts::PI;

fn tilted() -> Direction3 {
    Direction3::new(1.0, 1.0, 1.0).unwrap()
}

#[test]
fn axis1_relations_and_motions() {
    let axis = Axis1::new(Point3::new(1.0, 0.0, 0.0), Direction3::Z);
    let shifted = axis.translated(Vector3::new(0.0, 0.0, 5.0));
    assert!(axis.is_coaxial(shifted, 1e-12, 1e-12));
    assert!(!axis.is_coaxial(shifted.reversed(), 1e-12, 1e-12));
    assert!(axis.is_opposite(shifted.reversed(), 1e-12));
    assert!(axis.is_parallel(shifted.reversed(), 1e-12));
    assert!(axis.is_normal(Axis1::new(Point3::default(), Direction3::X), 1e-12));
    assert_eq!(axis.angle(shifted.reversed()), PI);
    let mirrored = axis.mirrored_point(Point3::default());
    assert_eq!(mirrored.location(), Point3::new(-1.0, 0.0, 0.0));
    assert_eq!(mirrored.direction(), Direction3::Z.reversed());
    let about_x = axis.mirrored_axis(Axis1::new(Point3::default(), Direction3::X));
    assert_eq!(about_x.direction().z(), -1.0);
    let plane = Axis2::new(Point3::default(), Direction3::Z).unwrap();
    assert_eq!(axis.mirrored_plane(plane).direction().z(), -1.0);
    let rotated = axis
        .rotated(Axis1::new(Point3::default(), Direction3::Z), PI)
        .unwrap();
    assert!(rotated.location().distance(&Point3::new(-1.0, 0.0, 0.0)) < 1e-15);
    assert_eq!(axis.scaled(Point3::default(), -1.0).direction().z(), -1.0);
    let moved = axis.translated_between(Point3::default(), Point3::new(0.0, 2.0, 0.0));
    assert_eq!(moved.location(), Point3::new(1.0, 2.0, 0.0));
    let turn = Transform3::rotation(Axis1::default(), PI / 2.0).unwrap();
    assert!(
        axis.transformed(turn)
            .location()
            .distance(&Point3::new(0.0, 1.0, 0.0))
            < 1e-15
    );
}

#[test]
fn axis1_json_round_trips() {
    let axis = Axis1::new(Point3::new(1.0, 2.0, 3.0), Direction3::Y);
    let mut stream = DumpStream::default();
    axis.dump_json(&mut stream).unwrap();
    assert_eq!(
        stream.text(),
        "\"Location\": [1, 2, 3], \"Direction\": [0, 1, 0]"
    );
    let mut parsed = Axis1::default();
    let mut position = 1;
    assert!(parsed.init_from_json(stream.text(), &mut position).unwrap());
    assert_eq!(parsed, axis);
}

#[test]
fn axis2_set_direction_swaps_collinear_directions() {
    let frame = Axis2::with_x_direction(Point3::default(), Direction3::Z, Direction3::X).unwrap();
    // New normal equal to X: X takes the old Y and Y the old normal.
    let mut same = frame;
    same.set_direction(Direction3::X).unwrap();
    assert_eq!(
        (same.x_direction(), same.y_direction()),
        (Direction3::Y, Direction3::Z)
    );
    // New normal opposite to X: X takes the old normal and Y is kept.
    let mut opposite = frame;
    opposite.set_direction(Direction3::X.reversed()).unwrap();
    assert_eq!(
        (opposite.x_direction(), opposite.y_direction()),
        (Direction3::Z, Direction3::Y)
    );
    let mut generic = frame;
    generic
        .set_axis(Axis1::new(Point3::new(1.0, 1.0, 1.0), tilted()))
        .unwrap();
    assert_eq!(generic.location(), Point3::new(1.0, 1.0, 1.0));
    assert!(generic.x_direction().is_normal(tilted(), 1e-15));
    let mut invalid = frame;
    assert!(invalid.set_x_direction(Direction3::Z).is_err());
    invalid = frame;
    invalid.set_y_direction(Direction3::X).unwrap();
    assert_eq!(invalid.x_direction(), Direction3::Y.reversed());
}

#[test]
fn axis2_coplanarity_and_symmetries() {
    let frame = Axis2::with_x_direction(Point3::default(), Direction3::Z, Direction3::X).unwrap();
    let lifted = frame.translated(Vector3::new(3.0, 0.0, 0.0));
    assert!(frame.is_coplanar(lifted, 1e-12, 1e-12));
    assert!(frame.is_coplanar_axis(
        Axis1::new(Point3::new(5.0, 5.0, 0.0), Direction3::X),
        1e-12,
        1e-12
    ));
    assert!(!frame.is_coplanar(frame.translated(Vector3::new(0.0, 0.0, 1.0)), 1e-12, 1e-12));
    assert_eq!(frame.angle(lifted), 0.0);
    let mirrored = frame
        .mirrored_plane(Axis2::new(Point3::default(), Direction3::X).unwrap())
        .unwrap();
    assert_eq!(mirrored.x_direction().x(), -1.0);
    assert_eq!(mirrored.direction().z(), -1.0);
    let about_x = frame
        .mirrored_axis(Axis1::new(Point3::default(), Direction3::X))
        .unwrap();
    assert_eq!(about_x.direction().z(), -1.0);
    let mut stream = DumpStream::default();
    frame.dump_json(&mut stream).unwrap();
    let mut parsed = Axis2::default();
    let mut position = 1;
    assert!(parsed.init_from_json(stream.text(), &mut position).unwrap());
    assert_eq!(parsed.direction(), frame.direction());
}

#[test]
fn axis3_right_handed_conversion_and_mirrors() {
    let mut frame = Axis3::new(Point3::default(), Direction3::Z, Direction3::X).unwrap();
    assert_eq!(frame.ax2().unwrap().direction(), Direction3::Z);
    frame.y_reverse();
    // An indirect frame converts to the right-handed frame with the reversed normal.
    assert_eq!(frame.ax2().unwrap().direction(), Direction3::Z.reversed());
    let mirrored = frame.mirrored_point(Point3::new(1.0, 0.0, 0.0));
    assert_eq!(mirrored.location(), Point3::new(2.0, 0.0, 0.0));
    assert_eq!(mirrored.direction(), Direction3::Z.reversed());
    let about = frame.mirrored_axis(Axis1::new(Point3::default(), Direction3::Y));
    assert_eq!(about.direction(), Direction3::Z.reversed());
    let moved = frame.translated_between(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 3.0, 4.0));
    assert_eq!(moved.location(), Point3::new(1.0, 2.0, 3.0));
    let reference =
        Axis3::from_axis2(Axis2::new(Point3::new(0.0, 0.0, 2.0), Direction3::Z).unwrap());
    assert!(!frame.is_coplanar(reference, 1e-12, 1e-12));
    assert!(frame.is_coplanar_axis(Axis1::new(Point3::default(), Direction3::X), 1e-12, 1e-12));
    assert_eq!(frame.angle(reference), 0.0);
    assert_eq!(frame.axis().direction(), Direction3::Z);
    let mut located = frame;
    located.set_location(Point3::new(0.0, 0.0, 7.0));
    assert_eq!(located.location().z(), 7.0);
}
