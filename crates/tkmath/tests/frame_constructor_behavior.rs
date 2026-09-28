//! Exact constructor distinction found by the quaternion displacement oracle.
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Plane3, Point3};
#[test]
fn automatic_ax3_preserves_candidate_while_ax2_and_plane_orthogonalize() {
    let normal =
        Direction3::new(-0.871754399063061, 1.2240802866824296, -9.383939787615201).unwrap();
    let origin = Point3::default();
    let frame = Axis3::from_normal(origin, normal).unwrap();
    let candidate = Direction3::new(0.0, normal.z(), -normal.y()).unwrap();
    assert_eq!(frame.x_direction().coordinates(), candidate.coordinates());
    let right_handed = Axis2::new(origin, normal).unwrap();
    let plane = Plane3::new(origin, normal).unwrap();
    assert_eq!(
        plane.position().x_direction().coordinates(),
        right_handed.x_direction().coordinates()
    );
    assert_ne!(
        frame.x_direction().coordinates(),
        right_handed.x_direction().coordinates()
    );
}
