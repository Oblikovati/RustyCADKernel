// SPDX-License-Identifier: GPL-2.0-only
//! Actual geometry and tolerance state remain unchanged by metric inspection.
use super::tolerance::inspect_planar_edge_residual;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Axis1, Axis2d, Direction2, Direction3, Point2};

use super::tolerance_test_support::{assert_exceeds, assert_within, inspect, metric_fixture};
use super::tolerance_types::{PlanarMetricCause, PlanarMetricInspection};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face},
};
use rustycad_tkg3d::geom::GeomBezierCurve;
use rustycad_tkmath::{
    gp::Point3,
    predicates::roots::{RootIsolationError, RootIsolationOptions, metric_bounds::ResidualError},
};
#[test]
fn stored_cubic_exact_residual_retains_affected_parent_kinds_and_limits() {
    let (edge, face) = metric_fixture(0.);
    let relation = super::source_relation_test_support::test_source_relation(&edge, &[11, 19]);
    let report = inspect_planar_edge_residual(
        &edge,
        &face,
        &[relation],
        0.,
        RootIsolationOptions::new(0.01),
        || false,
    );
    assert_within(&report);
    assert_contexts(&report, &edge, &face);
    assert_eq!(report.edge.source_parents, Some(vec![11, 19]));
    let parents = report.edge.original_parents.as_ref().unwrap();
    assert_eq!(parents.modified.len(), 2);
    assert_eq!(report.face.original_parents, None);
    assert_unchanged_limits(&report);
}
fn assert_contexts(report: &PlanarMetricInspection, edge: &Edge, face: &Face) {
    assert!(report.edge.shape.is_equal(edge));
    assert!(report.face.shape.is_equal(face));
}
#[test]
fn real_residual_exceedance_never_heals_or_widens_edge_tolerance() {
    let (edge, face) = metric_fixture(0.125);
    let tolerance = BRepTool::edge_tolerance(&edge).unwrap();
    assert_exceeds(&inspect(&edge, &face, 0.1, false));
    assert_eq!(BRepTool::edge_tolerance(&edge).unwrap(), tolerance);
    assert_within(&inspect(&edge, &face, 0.125, false));
}
#[test]
fn cancellation_retains_exact_cause_and_original_entities_without_mutation() {
    let (edge, face) = metric_fixture(0.);
    let tolerance = BRepTool::edge_tolerance(&edge).unwrap();
    let report = inspect(&edge, &face, 0., true);
    assert!(matches!(
        report.decision,
        Err(PlanarMetricCause::Exact(ResidualError::Roots(
            RootIsolationError::Cancelled
        )))
    ));
    assert_contexts(&report, &edge, &face);
    assert_eq!(BRepTool::edge_tolerance(&edge).unwrap(), tolerance);
    assert_within(&inspect(&edge, &face, 0., false));
}
#[test]
fn unknown_parameter_correspondence_is_incomplete_before_metric_execution() {
    let (edge, face) = metric_fixture(0.);
    BRepBuilder::default().same_parameter(&edge, false).unwrap();
    let report = inspect(&edge, &face, 0., false);
    assert!(matches!(
        report.decision,
        Err(PlanarMetricCause::MissingSameParameter)
    ));
}
#[test]
fn retained_coefficients_are_isolated_from_source_edits_after_validation() {
    use rustycad_tkmath::predicates::roots::metric_bounds::{
        ResidualDecision, certify_planar_residual,
    };
    let (edge, face) = metric_fixture(0.);
    let policy = RootIsolationOptions::new(0.01);
    let report = inspect(&edge, &face, 0., false);
    let stored = BRepTool::curve_with_location(&edge)
        .unwrap()
        .curve
        .cast::<GeomBezierCurve>();
    stored.set_pole(2, Point3::new(4., 0., 1.)).unwrap();
    let snapshot = report.coefficient_snapshot.unwrap();
    assert!(matches!(
        certify_planar_residual(snapshot.query(0., policy), || false).unwrap(),
        ResidualDecision::Within { .. }
    ));
    assert_exceeds(&inspect(&edge, &face, 0., false));
}

#[test]
fn actual_stored_line_uses_origin_direction_and_located_plane_coefficients() {
    let (edge, face) = actual_line_fixture();
    assert_within(&inspect(&edge, &face, 0., false));
    check_located_pair(&edge, &face);
}
fn check_located_pair(edge: &Edge, face: &Face) {
    use rustycad_tkmath::{gp::Transform3, toploc::Location};
    let location = Location::new(Transform3::translation([7., 11., -13.]));
    let moved_edge =
        rustycad_tkbrep::topods::edge(&edge.located(&location, false).unwrap()).unwrap();
    let moved_face =
        rustycad_tkbrep::topods::face(&face.located(&location, false).unwrap()).unwrap();
    assert_within(&inspect(&moved_edge, &moved_face, 0., false));
}
#[test]
fn mismatched_actual_ranges_keep_both_offending_domains_and_contexts() {
    let (edge, face) = metric_fixture(0.);
    BRepBuilder::default()
        .range_on_face(&edge, &face, 0., 0.75)
        .unwrap();
    let report = inspect(&edge, &face, 0., false);
    assert!(matches!(
        report.decision,
        Err(PlanarMetricCause::ParameterRanges {
            curve: [0., 1.],
            pcurve: [0., 0.75]
        })
    ));
    assert_contexts(&report, &edge, &face);
}
#[test]
fn actual_rational_curve_capture_preserves_nonunit_stored_weights() {
    use rustycad_tkg2d::geom2d::Geom2dBezierCurve;
    let (edge, face) = metric_fixture(0.);
    let space = BRepTool::curve_with_location(&edge)
        .unwrap()
        .curve
        .cast::<GeomBezierCurve>();
    let planar = BRepTool::curve_on_face(&edge, &face)
        .unwrap()
        .pcurve
        .cast::<Geom2dBezierCurve>();
    space.set_weight(2, 3.).unwrap();
    planar.set_weight(2, 3.).unwrap();
    assert!(space.is_rational() && planar.is_rational());
    assert_within(&inspect(&edge, &face, 0., false));
    assert_eq!(space.weight(2).unwrap(), 3.);
    assert_eq!(planar.weight(2).unwrap(), 3.);
}

fn assert_unchanged_limits(report: &PlanarMetricInspection) {
    assert_eq!(report.requested_spatial_limit, 0.);
    assert_eq!(report.effective_spatial_limit, 0.);
    assert_eq!(report.root_policy.max_width, 0.01);
    assert_eq!(report.failed_subdivision_work, None);
}

fn actual_line_fixture() -> (Edge, Face) {
    let space: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    let planar: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::new(1., 0.).unwrap(),
    )))
    .cast();
    let edge = rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_curve_params(
        &space, 0., 1.,
    )
    .unwrap()
    .edge()
    .unwrap();
    let face = super::tolerance_test_support::attach_planar_curve(&edge, &planar);
    (edge, face)
}

#[test]
fn actual_guarded_native_failure_keeps_cause_and_context_without_panic_relabeling() {
    let (_, face) = metric_fixture(0.);
    let edge = Edge::default();
    let report = inspect(&edge, &face, 0., false);
    let Err(PlanarMetricCause::Native(cause)) = report.decision else {
        panic!("expected original guarded failure");
    };
    assert_eq!(cause.exception_type(), "Standard_NullObject");
    assert!(report.edge.shape.is_null());
    assert!(report.face.shape.is_equal(&face));
    assert!(report.coefficient_snapshot.is_none());
}
