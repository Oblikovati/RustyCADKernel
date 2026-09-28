//! Frame updates away from collinearity and translated affine origins.
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Direction3, GeneralTransform3, Matrix3, Point3, Xyz,
};
#[test]
fn noncollinear_frame_updates_preserve_handedness() {
    for reverse in [false, true] {
        let mut frame = Axis3::default();
        if reverse {
            frame.z_reverse();
        }
        let direction = Direction3::new(1.0, 2.0, 3.0).unwrap();
        let expected = frame.direct().unwrap();
        frame.set_direction(direction).unwrap();
        assert_eq!(frame.direct().unwrap(), expected);
        frame.set_x_direction(Direction3::Y).unwrap();
        assert_eq!(frame.direct().unwrap(), expected);
        frame.set_y_direction(Direction3::X).unwrap();
        assert_eq!(frame.direct().unwrap(), expected);
        assert!(frame.x_direction().dot(frame.y_direction()).abs() < 1e-12);
        assert!(frame.x_direction().dot(frame.direction()).abs() < 1e-12);
    }
}
#[test]
fn axis_and_plane_affinities_respect_translated_origins() {
    let origin = Point3::new(1.0, 2.0, 3.0);
    let axis = Axis1::new(origin, Direction3::Z);
    let mut transform = GeneralTransform3::default();
    transform.set_affinity_axis(axis, 2.0);
    assert_eq!(
        transform.apply(Xyz::new(2.0, 3.0, 4.0)),
        Xyz::new(3.0, 4.0, 4.0)
    );
    let plane = Axis2::new(origin, Direction3::Z).unwrap();
    transform.set_affinity_plane(plane, 2.0);
    assert_eq!(
        transform.apply(Xyz::new(2.0, 3.0, 4.0)),
        Xyz::new(2.0, 3.0, 5.0)
    );
    let transform = GeneralTransform3::new(Matrix3::default(), Xyz::new(1.0, 2.0, 3.0));
    assert_eq!(
        transform.apply(Xyz::new(9.0, 8.0, 7.0)),
        Xyz::new(1.0, 2.0, 3.0)
    );
}
