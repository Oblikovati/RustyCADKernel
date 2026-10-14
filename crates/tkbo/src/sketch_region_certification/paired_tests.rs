// SPDX-License-Identifier: GPL-2.0-only
//! Independent analytic expected contacts exercise the actual owned-source census boundary.
use super::paired_types::*;
use super::source_binding_tests::source_policy;
use super::tests::{capture, policy};
use super::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::predicates::roots::bezier_pair::*;

pub(super) fn paired(shapes: &[Shape]) -> BoundaryCensus {
    enumerate_source_bound_census(capture(shapes), policy(), source_policy(), || false)
        .unwrap_or_else(|_| panic!("expected retained paired census"))
}
pub(super) fn bezier_shape(controls: &[[f64; 2]], weights: Option<&[f64]>) -> Shape {
    use rustycad_tkernel::{ncollection::Array1, standard::Handle};
    use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let points = control_points(controls);
    let poles = Array1::from_shared(&points, 1);
    let curve = match weights {
        Some(weights) => GeomBezierCurve::with_weights(&poles, &Array1::from_shared(weights, 1)),
        None => GeomBezierCurve::new(&poles),
    }
    .unwrap();
    let curve: Handle<dyn GeomCurve> = Handle::new(curve).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
pub(super) fn curved_pair() -> Vec<Shape> {
    vec![
        bezier_shape(&[[0., 0.], [0.5, 0.], [1., 1.]], None),
        bezier_shape(&[[0., 0.5], [0.5, 0.5], [1., -0.5]], None),
    ]
}
pub(super) fn distinct_record(census: &BoundaryCensus) -> &PairedStoredReport {
    census
        .paired
        .as_ref()
        .unwrap()
        .reports
        .iter()
        .find(|report| report.mode == PairedQueryMode::Distinct)
        .unwrap()
}
#[test]
fn actual_closed_cubic_self_uses_general_provider_and_retains_remaining_obligations() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let census = paired(&[source]);
    let proof = super::paired_binding::domain_certificate(&census, &census.domains[0]).unwrap();
    assert_eq!(proof.contact_count(), 1);
    let contact = proof.contact(0).unwrap();
    assert_eq!(contact.point, [0., 0.]);
    assert_eq!(contact.paired_parameter_bounds, [1., 1.]);
    assert!(
        !census
            .current_obligations()
            .contains(&CertificationObligation::ExhaustivePairAndSelfDomains)
    );
    assert_remaining(&census);
}
fn assert_remaining(census: &BoundaryCensus) {
    for gap in [
        CertificationObligation::MappedSplitCorrespondence,
        CertificationObligation::CandidateSearchAndCoverage,
        CertificationObligation::SelfIntersection,
        CertificationObligation::Tolerance,
    ] {
        assert!(census.current_obligations().contains(&gap));
    }
}
#[test]
fn curved_pair_keeps_original_root_and_paired_parameter_evidence() {
    let census = paired(&curved_pair());
    assert_eq!(census.domains.len(), 3);
    let report = distinct_record(&census).outcome.as_ref().unwrap();
    let proof = report.complete().unwrap();
    let contact = proof.contact(0).unwrap();
    assert_eq!(contact.point, [0.5, 0.25]);
    assert_eq!(contact.paired_parameter_bounds, [0.5, 0.5]);
    assert_eq!(
        contact.parameter.bounds().0.compare_f64(0.5).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_full_graph_evidence(&proof);
}
#[test]
fn generic_crossing_and_cusp_self_domains_have_independent_expected_dispositions() {
    let crossing = bezier_shape(&[[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]], None);
    let census = paired(&[crossing]);
    assert_crossing_contact(&census);
    let cusp = bezier_shape(&[[3., -3.], [-1., 3.], [-1., -3.], [3., 3.]], None);
    let census = paired(&[cusp]);
    let report = paired_reports(&census)[0].outcome.as_ref().unwrap();
    assert_eq!(report.complete().unwrap().contact_count(), 0);
}
#[test]
fn rational_and_polynomial_sources_share_the_same_domain_consumer() {
    let shapes = [
        bezier_shape(&[[0., 0.], [0.5, 1.]], Some(&[1., 2.])),
        bezier_shape(&[[0., 1.], [1., 0.]], None),
    ];
    let census = paired(&shapes);
    let report = distinct_record(&census).outcome.as_ref().unwrap();
    assert_eq!(
        report.source().captured_curves()[0].weights.unwrap(),
        [1., 2.]
    );
    let contact = report.complete().unwrap().contact(0).unwrap();
    assert!((contact.point[0] - 1. / 3.).abs() <= contact.achieved_world_error);
    assert!((contact.point[1] - 2. / 3.).abs() <= contact.achieved_world_error);
    assert_remaining(&census);
}
#[test]
fn positive_dimensional_overlap_retains_typed_failure_not_an_empty_certificate() {
    let controls = [[0., 0.], [0.5, 0.], [1., 1.]];
    let census = paired(&[bezier_shape(&controls, None), bezier_shape(&controls, None)]);
    let report = distinct_record(&census).outcome.as_ref().unwrap();
    assert_eq!(report.completion(), BezierPairCompletion::Incomplete);
    assert!(report.failure().is_some() && report.complete().is_none());
    assert!(
        census
            .current_obligations()
            .contains(&CertificationObligation::ExhaustivePairAndSelfDomains)
    );
}
#[test]
fn paired_evidence_is_not_interpreted_by_the_one_axis_approximate_inspector() {
    let shapes = curved_pair();
    let census = paired(&shapes);
    let arranged = crate::sketch_region_candidates::tests::arranged(&shapes);
    assert!(
        inspect_contact_coverage(&census, &arranged, rustycad_tkmath::gp::Plane3::default())
            .is_err()
    );
    assert!(census.reports.is_empty() && census.failures.is_empty());
}

fn control_points(controls: &[[f64; 2]]) -> Vec<rustycad_tkmath::gp::Point3> {
    controls
        .iter()
        .map(|point| rustycad_tkmath::gp::Point3::new(point[0], point[1], 0.))
        .collect()
}
pub(super) fn paired_reports(census: &BoundaryCensus) -> &[PairedStoredReport] {
    let evidence = census.paired.as_ref().unwrap();
    &evidence.reports
}
pub(super) fn has_obligation(census: &BoundaryCensus, gap: CertificationObligation) -> bool {
    let current = census.current_obligations();
    current.contains(&gap)
}
pub(super) fn complete_distinct(census: &BoundaryCensus) -> BezierPairCertificate<'_> {
    let report = distinct_record(census).outcome.as_ref().unwrap();
    report.complete().unwrap()
}
pub(super) fn retained_source(census: &BoundaryCensus) -> CapturedBezierPair {
    let report = distinct_record(census).outcome.as_ref().unwrap();
    report.source().clone()
}

fn assert_full_graph_evidence(proof: &BezierPairCertificate<'_>) {
    assert!(
        proof
            .denominators()
            .iter()
            .all(|proof| proof.roots().is_empty())
    );
    assert_eq!(
        proof.accepted_candidates().len(),
        proof.candidates().1.roots().len()
    );
}

fn assert_crossing_contact(census: &BoundaryCensus) {
    let proof = paired_reports(census)[0]
        .outcome
        .as_ref()
        .unwrap()
        .complete()
        .unwrap();
    assert_eq!(proof.contact(0).unwrap().point, [0.75, 0.]);
    assert_eq!(
        proof.contact(0).unwrap().paired_parameter_bounds,
        [0.75, 0.75]
    );
}
