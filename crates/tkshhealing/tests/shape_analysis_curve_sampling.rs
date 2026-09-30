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
