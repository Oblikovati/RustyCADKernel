//! Native sampling rules and overload side effects at the actual Graphic3d callback seam.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[path = "support/stdprs_curve_geometry.rs"]
mod geometry;
use display_driver::{DisplayTrace, SensitiveDisplayDriver};
use geometry::curve_geometry;
use rustycad_tkernel::{ncollection::Sequence, precision, standard::Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsCurve, v3d::V3dViewer};
fn fixture() -> (
    Handle<dyn Graphic3dStructureSource>,
    Handle<Prs3dDrawer>,
    DisplayTrace,
) {
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
        .unwrap()
        .cast();
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.set_maximal_parameter_value(12.);
    drawer.set_discretisation(4);
    (prs, drawer, trace)
}
#[test]
fn sample_sequences_append_and_use_native_knot_scaling() {
    let (prs, drawer, _) = fixture();
    for (kind, count) in [(0, 2), (1, 4), (2, 4), (3, 4), (4, 12)] {
        let curve = curve_geometry(kind).unwrap();
        let mut points = Sequence::default();
        points.append(Point3::new(9., 8., 7.));
        StdPrsCurve::add_points(&prs, &curve, &drawer, &mut points, true).unwrap();
        assert_eq!(points.len(), count + 1);
        assert_eq!(points.iter().next().unwrap().x(), 9.);
        assert_eq!(
            *points.iter().nth(1).unwrap(),
            curve.value(curve.first_parameter().unwrap()).unwrap()
        );
    }
}
#[test]
fn automatic_limits_expand_but_explicit_limits_clamp() {
    let (prs, drawer, _) = fixture();
    for (kind, expected) in [(5, [-8., 8.]), (6, [-13., 3.]), (7, [2., 18.])] {
        let curve = curve_geometry(kind).unwrap();
        let mut points = Sequence::default();
        StdPrsCurve::add_points(&prs, &curve, &drawer, &mut points, false).unwrap();
        assert_eq!(
            *points.iter().next().unwrap(),
            curve.value(expected[0]).unwrap()
        );
        assert_eq!(
            *points.iter().last().unwrap(),
            curve.value(expected[1]).unwrap()
        );
    }
    let curve = curve_geometry(5).unwrap();
    let query = curve.value(11.).unwrap();
    assert!(!StdPrsCurve::matches(&query, 0.01, &curve, &drawer).unwrap());
    assert!(
        StdPrsCurve::matches_range(
            &query,
            0.01,
            &curve,
            -precision::infinite(),
            precision::infinite(),
            &drawer
        )
        .unwrap()
    );
}
#[test]
fn point_overloads_keep_aspect_and_suppress_arrows_even_when_enabled() {
    let (prs, drawer, trace) = fixture();
    drawer.set_line_arrow_draw(true);
    let curve = curve_geometry(0).unwrap();
    prs.current_group().unwrap();
    trace.write().events.clear();
    StdPrsCurve::add_points(&prs, &curve, &drawer, &mut Sequence::default(), false).unwrap();
    assert!(trace.read().events.is_empty());
    StdPrsCurve::add(&prs, &curve, &drawer, false).unwrap();
    assert!(trace.read().events.starts_with(&[3.]));
    assert!(trace.read().events.windows(3).any(|w| w
        == [
            4.,
            Graphic3dTypeOfPrimitiveArray::Segments as i32 as f64,
            16.
        ]));
}
#[test]
fn matching_preserves_manhattan_endpoint_and_strict_segment_thresholds() {
    let curve = curve_geometry(0).unwrap();
    let (prs, drawer, _) = fixture();
    StdPrsCurve::add_range(&prs, &curve, 0., 1., &drawer, true).unwrap();
    let endpoint = Point3::new(-3., -1.5, 0.);
    let middle = Point3::new(0., -1.5, 0.);
    assert!(StdPrsCurve::matches(&endpoint, 0.5, &curve, &drawer).unwrap());
    assert!(!StdPrsCurve::matches_with_sampling(&middle, 0.5, &curve, 0.01, 12., 4).unwrap());
    assert!(
        StdPrsCurve::matches_range_with_sampling(&middle, 0.5001, &curve, 0., 6., 0.01, 4).unwrap()
    );
}
#[test]
fn circle_matching_can_overshoot_the_requested_last_parameter() {
    let curve = curve_geometry(1).unwrap();
    let step = (8_f64 * 0.3 / 3.).sqrt();
    let query = curve.value(2. * step).unwrap();
    assert!(
        StdPrsCurve::matches_range_with_sampling(&query, 1e-10, &curve, 0., 0.05, 0.3, 30).unwrap()
    );
    let (prs, _, _) = fixture();
    let mut points = Sequence::default();
    StdPrsCurve::add_range_points(&prs, &curve, 0., 0.05, &mut points, 1, false).unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(*points.iter().last().unwrap(), curve.value(0.05).unwrap());
}
