//! OCC23361: composed and sequential transforms must produce equal points.
#![allow(non_snake_case, clippy::excessive_precision)]
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};
#[test]
fn gp_TrsfTest__OCC23361_TransformationComposition() {
    let origin = Point3::new(0.0, 0.0, 2.0);
    let first =
        Transform3::rotation(Axis1::new(origin, Direction3::Y), -0.49328285294022267).unwrap();
    let second =
        Transform3::rotation(Axis1::new(origin, Direction3::Z), 0.87538474718473880).unwrap();
    let composed = second.multiplied(first);
    let point = Point3::new(10.0, 3.0, 4.0);
    let combined_point = point.transformed(composed);
    let mut sequential_point = point.transformed(first);
    sequential_point.transform(second);
    assert!(combined_point.is_equal(&sequential_point, 1e-7));
}
