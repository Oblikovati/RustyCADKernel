//! Sampling retains preexisting sequence values, including on invalid ranges.
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCurve;

fn sampling_line() -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::X,
    )))
    .cast()
}
#[test]
fn samples_append_both_line_endpoints_without_clearing_sequence() {
    let mut points = Sequence::default();
    points.append(Point2::new(8.0, 9.0));
    assert!(
        ShapeAnalysisCurve::get_sample_points_2d(&sampling_line(), 2.0, 5.0, &mut points).unwrap()
    );
    assert_eq!(
        points.iter().copied().collect::<Vec<_>>(),
        vec![
            Point2::new(8.0, 9.0),
            Point2::new(2.0, 0.0),
            Point2::new(5.0, 0.0)
        ]
    );
}
#[test]
fn reversed_range_preserves_existing_sequence() {
    let mut points = Sequence::default();
    points.append(Point2::new(8.0, 9.0));
    assert!(
        ShapeAnalysisCurve::get_sample_points_2d(&sampling_line(), 5.0, 2.0, &mut points).is_err()
    );
    assert_eq!(points.length(), 1);
    assert_eq!(*points.value(1).unwrap(), Point2::new(8.0, 9.0));
}
#[test]
fn zero_length_line_still_appends_two_equal_samples() {
    let mut points = Sequence::default();
    assert!(
        ShapeAnalysisCurve::get_sample_points_2d(&sampling_line(), 2.0, 2.0, &mut points).unwrap()
    );
    assert_eq!(points.length(), 2);
    assert_eq!(*points.value(1).unwrap(), *points.value(2).unwrap());
}

#[test]
fn offset_3d_sampling_uses_basis_points_instead_of_offset_geometry() {
    use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomOffsetCurve};
    use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
    let basis: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    let offset: Handle<dyn GeomCurve> =
        Handle::new(GeomOffsetCurve::new(&basis, 2.0, Direction3::Z, false).unwrap()).cast();
    let mut points = Sequence::default();
    assert!(ShapeAnalysisCurve::get_sample_points(&offset, 0.0, 1.0, &mut points).unwrap());
    assert_eq!(
        points.iter().copied().collect::<Vec<_>>(),
        vec![Point3::default(), Point3::new(1.0, 0.0, 0.0)]
    );
    assert_ne!(offset.value(0.0).unwrap(), *points.value(1).unwrap());
}
#[test]
fn zero_range_circle_sampling_appends_only_last_point() {
    use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
    use rustycad_tkmath::gp::Axis2;
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 2.0).unwrap()).cast();
    let mut points = Sequence::default();
    assert!(ShapeAnalysisCurve::get_sample_points(&circle, 0.5, 0.5, &mut points).unwrap());
    assert_eq!(points.length(), 1);
    assert_eq!(*points.value(1).unwrap(), circle.value(0.5).unwrap());
}
