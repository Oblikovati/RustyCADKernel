//! Contap walking-line runtime identity and mutation contracts.
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
use rustycad_tkhlr::contap::ContapTheIWLineOfTheIWalking as Line;
use rustycad_tkmath::gp::Point3;
fn sample_point(x: f64) -> IntSurfPntOn2S {
    let mut point = IntSurfPntOn2S::new();
    point.set_value_all(Point3::new(x, 0., 0.), x, 0., x, 0.);
    point
}
#[test]
fn contour_runtime_identity_is_distinct_from_intpatch() {
    let line = Handle::new(Line::new(None));
    assert_eq!(
        line.dynamic_type().unwrap().name(),
        "Contap_TheIWLineOfTheIWalking"
    );
    assert_eq!(line.nb_points(), 0);
    assert!(!line.has_first_point());
    assert!(line.first_point().is_err());
    assert!(line.last_point().is_err());
    assert!(line.value(0).is_err());
    assert!(line.passing_point(1).is_err());
}
#[test]
fn line_handle_shares_points_and_reverse_preserves_endpoint_flags() {
    let line = Line::new(None);
    line.add_point(&sample_point(1.));
    line.add_point(&sample_point(2.));
    let points = line.line();
    line.add_point(&sample_point(3.));
    assert_eq!(points.nb_points(), 3);
    line.add_status_first_last(true, true, false);
    line.reverse();
    assert!(line.is_closed());
    assert!(line.has_first_point());
    assert!(!line.has_last_point());
    assert_eq!(line.value(1).unwrap().value(), Point3::new(3., 0., 0.));
    line.cut(2).unwrap();
    assert_eq!(line.nb_points(), 1);
}

#[test]
fn native_beginning_setter_changes_only_the_end_flag() {
    use rustycad_tkgeomalgo::int_walk::TheIWLine;
    let line = Line::new(None);
    line.set_tangency_at_begining(true);
    assert!(!line.is_tangent_at_begining());
    assert!(line.is_tangent_at_end());
    TheIWLine::set_tangency_at_begining(&line, false);
    assert!(!line.is_tangent_at_begining());
    assert!(!line.is_tangent_at_end());
}
