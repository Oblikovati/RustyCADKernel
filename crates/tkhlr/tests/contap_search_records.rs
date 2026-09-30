//! Boundary limits own copies and preserve native missing-limit failures.
use rustycad_tkhlr::contap::{
    ContapThePathPointOfTheSearch as Point, ContapTheSegmentOfTheSearch as Segment,
};
use rustycad_tkmath::gp::Point3;
#[test]
fn setting_a_segment_curve_clears_limits() {
    let mut segment = Segment::new();
    assert!(!segment.has_first_point());
    assert!(segment.first_point().is_err());
    let mut point = Point::on_arc(Point3::new(1., 2., 3.), 1e-7, &Default::default(), 0.5);
    segment.set_limit_point(&point, true);
    point.set_value_on_arc(Point3::default(), 2e-7, &Default::default(), 0.75);
    assert_eq!(segment.first_point().unwrap().parameter(), 0.5);
    segment.set_limit_point(&point, false);
    assert_eq!(segment.last_point().unwrap().parameter(), 0.75);
    segment.set_value(&Default::default());
    assert!(segment.first_point().is_err());
    assert!(segment.last_point().is_err());
    assert!(segment.curve().is_null());
}
#[test]
fn vertex_state_tracks_the_selected_constructor_and_setter() {
    let mut point = Point::new();
    assert!(point.is_new());
    assert!(point.vertex().is_err());
    point.set_value_on_vertex(
        Point3::new(1., 2., 3.),
        1e-7,
        &Default::default(),
        &Default::default(),
        0.5,
    );
    assert!(!point.is_new());
    assert!(point.vertex().unwrap().is_null());
    assert_eq!(point.tolerance(), 1e-7);
    point.set_value_on_arc(Point3::default(), 2e-7, &Default::default(), 0.75);
    assert!(point.is_new());
    assert!(point.vertex().is_err());
    assert!(point.arc().is_null());
    assert_eq!(point.value(), Point3::default());
}
