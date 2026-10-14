// SPDX-License-Identifier: GPL-2.0-only
//! Degenerate domains use independent expected roots through the same actual-source consumer.
use super::paired_tests::*;
use super::source_binding_tests::source_policy;
use super::tests::{capture, policy};
use super::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::predicates::roots::bezier_pair::*;
#[test]
fn complete_distinct_empty_retains_original_candidates_excluded_by_actual_trim() {
    let mut shapes = curved_pair();
    shapes[1] = trimmed_edge(&shapes[1], [0.75, 1.]);
    let census = paired(&shapes);
    let proof = complete_distinct(&census);
    assert_eq!(proof.contact_count(), 0);
    assert_eq!(proof.candidates().1.roots().len(), 1);
    assert_eq!(proof.accepted_candidates(), [false]);
    assert_eq!(
        distinct_record(&census)
            .outcome
            .as_ref()
            .unwrap()
            .source()
            .captured_curves()[1]
            .interval,
        [0.75, 1.]
    );
}
fn trimmed_edge(shape: &Shape, bounds: [f64; 2]) -> Shape {
    use rustycad_tkbrep::{brep::BRepTool, topods::edge};
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let (curve, _, _) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
    BRepBuilderAPIMakeEdge::from_curve_params(&curve, bounds[0], bounds[1])
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
#[test]
fn tangency_retains_double_root_instead_of_point_proximity_or_duplicate_contacts() {
    let census = paired(&tangent_pair(0.));
    let proof = complete_distinct(&census);
    assert_eq!(proof.contact_count(), 1);
    let contact = proof.contact(0).unwrap();
    assert_eq!(contact.parameter.multiplicity(), 2);
    assert_eq!(contact.point, [0.5, 0.]);
    assert_eq!(contact.paired_parameter_bounds, [0.5, 0.5]);
    assert!(has_obligation(
        &census,
        CertificationObligation::MappedSplitCorrespondence
    ));
}
fn tangent_pair(height: f64) -> Vec<Shape> {
    vec![
        bezier_shape(&[[0., 1.], [0.5, -1.], [1., 1.]], None),
        bezier_shape(&[[0., height], [1., height]], None),
    ]
}
#[test]
fn near_tangent_binary_perturbations_preserve_exact_two_or_zero_root_dispositions() {
    let offset = 2_f64.powi(-50);
    let positive = paired(&tangent_pair(offset));
    let proof = complete_distinct(&positive);
    assert_eq!(proof.contact_count(), 2);
    let step = 2_f64.powi(-26);
    assert_eq!(proof.contact(0).unwrap().point, [0.5 - step, offset]);
    assert_eq!(proof.contact(1).unwrap().point, [0.5 + step, offset]);
    let negative = paired(&tangent_pair(-offset));
    assert_eq!(complete_distinct(&negative).contact_count(), 0);
}
#[test]
fn shared_endpoint_stays_exact_closed_domain_contact_not_an_omitted_pair() {
    let shapes = [
        bezier_shape(&[[0., 0.], [0.5, 0.], [1., 1.]], None),
        bezier_shape(&[[1., 1.], [1.5, 1.], [2., 2.]], None),
    ];
    let census = paired(&shapes);
    let contact = complete_distinct(&census).contact(0).unwrap();
    assert_eq!(contact.point, [1., 1.]);
    assert_eq!(
        contact.parameter.bounds().0.compare_f64(1.).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(contact.paired_parameter_bounds, [0., 0.]);
}
#[test]
fn actual_exceptional_fibre_preserves_original_typed_cause_and_whole_domain_gap() {
    let shapes = [
        bezier_shape(&[[0., 0.], [0.5, 0.], [1., 1.]], None),
        bezier_shape(&[[0.5, 0.], [0.5, 1.]], Some(&[1., 2.])),
    ];
    let census = paired(&shapes);
    let report = distinct_record(&census).outcome.as_ref().unwrap();
    let Some(BezierPairFailure::Operation(cause)) = report.failure() else {
        panic!("expected typed exceptional fibre");
    };
    assert!(matches!(
        cause.original_cause(),
        BezierPairOperationFailure::ExceptionalFiber(_)
    ));
    assert!(report.complete().is_none());
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
}
#[test]
fn immutable_captured_failure_names_actual_source_slot_and_keeps_raw_coefficients() {
    let shapes = curved_pair();
    let mut inputs = capture(&shapes);
    inputs.segments[0].controls[1][0] = f64::INFINITY;
    let census = enumerate_source_bound_census(inputs, policy(), source_policy(), || false)
        .unwrap_or_else(|_| panic!("expected retained invalid capture records"));
    let paired = census.paired.as_ref().unwrap();
    assert!(paired.occurrences[0].source.controls[1][0].is_infinite());
    assert_nonfinite_source_report(distinct_record(&census));
    assert!(has_obligation(
        &census,
        CertificationObligation::SourceConversionAndPlane
    ));
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
}
#[test]
fn binary_scales_and_axis_swaps_keep_source_bound_contacts_and_current_history() {
    for scale in [0.125, 1., 8.] {
        for swapped in [false, true] {
            verify_scaled_pair(scale, swapped);
        }
    }
}
fn verify_scaled_pair(scale: f64, swapped: bool) {
    let first = [[0., 0.], [0.5, 0.], [1., 1.]].map(|point| transformed(point, scale, swapped));
    let second =
        [[0., 0.5], [0.5, 0.5], [1., -0.5]].map(|point| transformed(point, scale, swapped));
    let census = paired(&[bezier_shape(&first, None), bezier_shape(&second, None)]);
    assert_eq!(
        complete_distinct(&census).contact(0).unwrap().point,
        transformed([0.5, 0.25], scale, swapped)
    );
    assert!(has_obligation(
        &census,
        CertificationObligation::SelfIntersection
    ));
}
fn transformed(mut point: [f64; 2], scale: f64, swapped: bool) -> [f64; 2] {
    if swapped {
        point.swap(0, 1);
    }
    point.map(|coordinate| coordinate * scale)
}

fn assert_nonfinite_source_report(record: &super::paired_types::PairedStoredReport) {
    let report = record.outcome.as_ref().unwrap();
    assert_eq!(report.completion(), BezierPairCompletion::Invalid);
    let Some(BezierPairFailure::Operation(cause)) = report.failure() else {
        panic!("expected original source conversion failure");
    };
    let BezierPairOperationFailure::SourceDomain { source_index, .. } = cause else {
        panic!("expected source ordinal");
    };
    assert_eq!(*source_index, 0);
    let BezierPairOperationFailure::Source(BezierPairCaptureFailure::Coordinate(coordinate)) =
        cause.original_cause()
    else {
        panic!("expected original nonfinite coordinate");
    };
    assert_eq!((coordinate.point, coordinate.axis), (1, 0));
    assert!(coordinate.value.is_infinite());
    assert!(report.complete().is_none());
}

#[test]
fn canonical_parameter_slots_preserve_reversed_alias_axes_with_distinct_parameters() {
    let first = bezier_shape(&[[0., 0.], [0.5, 1.]], Some(&[1., 2.]));
    let second = bezier_shape(&[[0., 1.], [1., 0.]], None);
    let census = paired(&[first.clone(), second, first.reversed()]);
    let domain = &census.domains[4];
    let DomainEvidence::PairedReport {
        source_slots,
        reused: true,
        ..
    } = domain.evidence
    else {
        panic!("expected exact alias reuse with source-axis binding");
    };
    assert_eq!(domain.occurrences, [1, 2]);
    assert_eq!(source_slots, [1, 0]);
    assert_alias_parameter_values(&census, domain);
}

fn assert_alias_parameter_values(census: &BoundaryCensus, domain: &BoundaryDomain) {
    let proof = super::paired_binding::domain_certificate(census, domain).unwrap();
    assert_eq!(
        proof
            .contact(0)
            .unwrap()
            .parameter
            .bounds()
            .0
            .compare_f64(0.5)
            .unwrap(),
        std::cmp::Ordering::Equal
    );
    assert!(proof.contact(0).unwrap().paired_parameter_bounds[0] < 0.5);
}
#[test]
fn wrong_capture_weight_count_keeps_typed_capture_failure_and_zero_spent_work() {
    let mut inputs = capture(&curved_pair());
    inputs.segments[0].weights = Some(vec![1., 2.]);
    let census = enumerate_source_bound_census(inputs, policy(), source_policy(), || false)
        .unwrap_or_else(|_| panic!("expected retained malformed source report"));
    let failure = distinct_record(&census).outcome.as_ref().err().unwrap();
    let BezierPairCaptureFailure::Curve { curve_index, cause } = failure.as_ref() else {
        panic!("expected capture source ordinal");
    };
    assert_eq!(*curve_index, 0);
    assert!(matches!(
        cause.as_ref(),
        BezierPairCaptureFailure::WeightCount {
            controls: 3,
            weights: 2
        }
    ));
    assert_eq!(distinct_record(&census).completed_work(), 0);
}
#[test]
fn wrong_parameter_slot_mapping_invalidates_current_binding_without_changing_exact_roots() {
    let mut census = paired(&curved_pair());
    let DomainEvidence::PairedReport { source_slots, .. } = &mut census.domains[1].evidence else {
        panic!("expected paired domain");
    };
    *source_slots = [1, 0];
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[1]).is_none());
    assert_eq!(
        complete_distinct(&census).contact(0).unwrap().point,
        [0.5, 0.25]
    );
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
}

#[test]
fn omitted_occurrence_capture_is_incomplete_without_indexing_a_missing_record() {
    let mut census = paired(&curved_pair());
    census.paired.as_mut().unwrap().occurrences.clear();
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[1]).is_none());
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
}
