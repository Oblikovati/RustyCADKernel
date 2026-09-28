//! Failure-state and mirror-overload differences preserved from pinned OCCT.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3, Xyz};
#[test]
fn failed_cross_keeps_native_zero_components() {
    let mut direction = Direction3::X;
    assert_eq!(
        direction.cross(Direction3::X).unwrap_err().message(),
        "gp_Dir::Cross() - result vector has zero norm"
    );
    assert_eq!(direction.coordinates(), &[0.0; 3]);
    direction = Direction3::X;
    assert!(direction.cross_cross(Direction3::X, Direction3::X).is_err());
    assert_eq!(direction.coordinates(), &[0.0; 3]);
}
#[test]
fn axis_mirror_preserves_native_y_for_z_bug() {
    let axis = Axis1::new(Point3::default(), Direction3::Y);
    let mut direction = Direction3::Z;
    direction.mirror_axis(axis);
    assert_eq!(direction.coordinates(), &[0.0, 2.0, 1.0]);
    let mut by_direction = Direction3::Z;
    by_direction.mirror_direction(Direction3::Y);
    assert_eq!(by_direction, Direction3::new(0.0, 0.0, -1.0).unwrap());
    assert_ne!(direction, by_direction);
    by_direction.reverse();
    assert_eq!(by_direction, Direction3::Z);
}
#[test]
fn direction_fragments_are_not_renormalized() {
    let mut direction = Direction3::default();
    let mut position = 1;
    assert!(
        direction
            .init_from_json("\"gp_Dir\": [0, 3, 4]", &mut position)
            .unwrap()
    );
    let mut stream = DumpStream::default();
    direction.dump_json(&mut stream).unwrap();
    assert_eq!(stream.text(), "\"gp_Dir\": [0, 3, 4]");
    assert_eq!(Direction3::X.dot_cross(Direction3::Y, Direction3::Z), 1.0);
    let frame = Axis2::new(Point3::default(), Direction3::Z).unwrap();
    assert_eq!(frame.location(), Point3::default());
    assert_eq!(frame.direction(), Direction3::Z);
    let axis = Axis1::default();
    let mut rotated = Direction3::X;
    rotated.rotate(axis, 0.0).unwrap();
    assert_eq!(rotated, Direction3::X);
    rotated.transform(Transform3::translation([1.0, 2.0, 3.0]));
    assert_eq!(rotated, Direction3::X);
    assert_eq!(Xyz::from(rotated), Xyz::new(1.0, 0.0, 0.0));
}
