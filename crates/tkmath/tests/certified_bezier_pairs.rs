// SPDX-License-Identifier: GPL-2.0-only
//! Public source-bound provider acceptance; no OCCT harness is required.
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_pair::*};
fn policy() -> BezierPairPolicy {
    BezierPairPolicy {
        version: 1,
        roots: RootIsolationOptions::new(0.01),
        signs: BezierPairSignOptions::derived(),
        requested_world_error: 1e-8,
    }
}
fn curve(controls: &[[f64; 2]]) -> BezierPairCurve<'_> {
    BezierPairCurve {
        controls,
        weights: None,
        interval: [0., 1.],
    }
}
fn report() -> BezierPairReport {
    let left = [[0., 0.], [0.5, 0.], [1., 1.]];
    let right = [[0., 1.], [0.5, 1.], [1., 0.]];
    let query = CapturedBezierPair::capture_pair(curve(&left), curve(&right)).unwrap();
    query.certify(policy(), || false)
}
#[test]
fn public_curved_pair_retains_sources_policies_and_complete_joint_coverage() {
    let report = report();
    assert!(report.failure().is_none());
    assert_eq!(report.policy(), policy());
    assert_eq!(report.source().captured_curves()[0].controls[2], [1., 1.]);
    let certificate = report.complete().unwrap();
    assert_eq!(certificate.work(), report.work());
    assert_eq!(certificate.captured_curves()[1].controls[0], [0., 1.]);
    assert_contact(&certificate);
    assert_coverage(&certificate);
}
fn assert_contact(certificate: &BezierPairCertificate<'_>) {
    assert_eq!(certificate.contact_count(), 1);
    assert!(certificate.contact(1).is_none());
    let contact = certificate.contact(0).unwrap();
    assert_eq!(contact.candidate_index, 0);
    assert!(contact.parameter.width().compare_f64(1e-8).unwrap().is_lt());
    assert!(contact.paired_parameter_bounds[0] <= contact.point[0]);
    assert!(contact.point[0] <= contact.paired_parameter_bounds[1]);
    assert!(contact.world_bounds[1][0] <= 0.5 && contact.world_bounds[1][1] >= 0.5);
    assert!(contact.achieved_world_error > 0. && contact.achieved_world_error <= 1e-8);
}
fn assert_coverage(certificate: &BezierPairCertificate<'_>) {
    assert_eq!(certificate.candidates().1.roots().len(), 1);
    assert!(certificate.candidates().0.polynomial_degree().is_some());
    assert_eq!(certificate.accepted_candidates(), &[true]);
    assert_eq!(certificate.membership().len(), 4);
    assert_membership(certificate);
    assert_regular_domains(certificate);
    assert_projection(certificate);
}
fn assert_membership(certificate: &BezierPairCertificate<'_>) {
    for predicate in certificate.membership() {
        assert!(predicate.rule.accepts(predicate.evaluation.signs()[0]));
        assert_eq!(predicate.evaluation.isolation(), certificate.candidates().1);
        let _exact_query = predicate.coefficients;
    }
}
fn assert_projection(certificate: &BezierPairCertificate<'_>) {
    let projection = certificate.projection();
    assert!(projection.alpha.coefficient_count() > 0);
    assert!(projection.beta.coefficient_count() > 0);
    assert_eq!(projection.cleared_originals.len(), 2);
    let leading = projection.alpha.signed_coefficient(0).unwrap().to_string();
    assert!(leading.parse::<i64>().unwrap() < 0);
    assert!(projection.alpha.signed_coefficient(5).is_none());
    assert_eq!(certificate.leading_factors().len(), 0);
    assert_eq!(certificate.homogeneous_sources().len(), 2);
    assert_eq!(certificate.effective_signs().requested, policy().signs);
}
#[test]
fn public_canonical_closed_cubic_self_survives_query_drop_and_input_mutation() {
    let mut controls = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let query = CapturedBezierPair::self_off_diagonal(&controls, None, [0., 1.]).unwrap();
    controls.fill([f64::NAN, f64::NAN]);
    let report = query.certify(policy(), || false);
    drop(query);
    let certificate = report.complete().unwrap();
    let contact = certificate.contact(0).unwrap();
    assert_eq!(certificate.contact_count(), 1);
    assert_eq!(
        contact.parameter.bounds().0.compare_f64(0.).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(contact.paired_parameter_bounds, [1., 1.]);
    assert_eq!(contact.point, [0., 0.]);
    assert_eq!(report.source().captured_curves()[0].controls[1], [4., 0.]);
}
#[test]
fn public_incomplete_report_retains_original_typed_cause_and_retry_is_clean() {
    let controls = [[0., 0.], [0.5, 0.], [1., 1.]];
    let query = CapturedBezierPair::capture_pair(curve(&controls), curve(&controls)).unwrap();
    let report = query.certify(policy(), || false);
    assert!(report.complete().is_none());
    assert_overlap_incomplete(&report);
    let mut unknown = policy();
    unknown.version = 77;
    let version = query.certify(unknown, || panic!("unknown policy must not poll"));
    assert!(matches!(
        version.failure(),
        Some(BezierPairFailure::PolicyVersion(77))
    ));
    assert_eq!(version.work().completed, 0);
    assert!(query.certify(policy(), || false).complete().is_none());
}

fn assert_overlap_incomplete(report: &BezierPairReport) {
    assert!(matches!(
        report.failure(),
        Some(BezierPairFailure::Operation(
            BezierPairOperationFailure::PositiveDimensional
        ))
    ));
}
#[test]
fn public_completion_separates_cancel_invalid_incomplete_and_empty_complete() {
    let controls = [[3., -3.], [-1., 3.], [-1., -3.], [3., 3.]];
    let query = CapturedBezierPair::self_off_diagonal(&controls, None, [0., 1.]).unwrap();
    let empty = query.certify(policy(), || false);
    assert_eq!(empty.completion(), BezierPairCompletion::Complete);
    assert_eq!(empty.complete().unwrap().contact_count(), 0);
    let cancelled = query.certify(policy(), || true);
    assert_eq!(cancelled.completion(), BezierPairCompletion::Cancelled);
    assert!(cancelled.complete().is_none());
    let mut invalid = policy();
    invalid.requested_world_error = f64::NAN;
    let failure = query.certify(invalid, || false);
    assert_eq!(failure.completion(), BezierPairCompletion::Invalid);
    assert!(
        matches!(failure.failure(), Some(BezierPairFailure::Operation(BezierPairOperationFailure::WorldRequest(value))) if value.is_nan())
    );
}
#[test]
fn public_capture_failure_includes_actual_offending_counts_and_expected_shape() {
    let controls = [[0., 0.], [1., 1.]];
    let bad = BezierPairCurve {
        controls: &controls,
        weights: Some(&[1.]),
        interval: [0., 1.],
    };
    for (left, right, index) in [(bad, curve(&controls), 0), (curve(&controls), bad, 1)] {
        let failure = CapturedBezierPair::capture_pair(left, right).unwrap_err();
        assert_capture_curve_counts(&failure, index);
        assert_capture_message(&failure);
    }

}

fn assert_regular_domains(certificate: &BezierPairCertificate<'_>) {
    assert!(
        certificate
            .denominators()
            .iter()
            .all(|roots| roots.roots().is_empty())
    );
    assert!(
        certificate
            .exceptional_fibers()
            .isolation()
            .roots()
            .is_empty()
    );
}
fn rational_query(swapped: bool, reversed_line: bool) -> CapturedBezierPair {
    let controls = [[0., 0.], [0.5, 1.]];
    let rational = BezierPairCurve { controls: &controls, weights: Some(&[1., 2.]), interval: [0., 1.] };
    let line = if reversed_line { [[1., 0.], [0., 1.]] } else { [[0., 1.], [1., 0.]] };
    if swapped { return CapturedBezierPair::capture_pair(curve(&line), rational).unwrap(); }
    CapturedBezierPair::capture_pair(rational, curve(&line)).unwrap()
}
#[test]
fn independent_nonsymmetric_pair_source_permutation_and_reversal_preserve_exact_contact() {
    let mut requested = policy();
    requested.requested_world_error = 5e-17;
    for (swapped, reversed) in [(false, false), (false, true), (true, false)] {
        let report = rational_query(swapped, reversed).certify(requested, || false);
        assert_eq!(report.completion(), BezierPairCompletion::Complete);
        assert_rational_correspondence(&report, swapped, reversed);
    }
}
fn assert_rational_correspondence(report: &BezierPairReport, swapped: bool, reversed: bool) {
    let proof = report.complete().unwrap();
    let contact = proof.contact(0).unwrap();
    assert_eq!(proof.contact_count(), 1);
    let expected = if swapped { 0.5 } else if reversed { 2. / 3. } else { 1. / 3. };
    assert!(contact.paired_parameter_bounds[0] <= expected && expected <= contact.paired_parameter_bounds[1]);
    assert_eq!(contact.point, [1. / 3., 2. / 3.]);
    assert!(contact.achieved_world_error <= 5e-17);
}
#[test]
fn distinct_tangent_endpoint_retains_actual_even_multiplicity_without_duplicate_contact() {
    let parabola = [[0., 0.], [0.5, 0.], [1., 1.]];
    let line = [[0., 0.], [1., 0.]];
    let query = CapturedBezierPair::capture_pair(curve(&parabola), curve(&line)).unwrap();
    let report = query.certify(policy(), || false);
    let proof = report.complete().unwrap();
    assert_eq!(proof.contact_count(), 1);
    let contact = proof.contact(0).unwrap();
    assert_eq!(contact.parameter.multiplicity(), 2);
    assert_eq!(contact.parameter.bounds().0.compare_f64(0.).unwrap(), std::cmp::Ordering::Equal);
    assert_eq!(contact.paired_parameter_bounds, [0., 0.]);
    assert_eq!(contact.point, [0., 0.]);
}

fn assert_capture_curve_counts(failure: &BezierPairCaptureFailure, expected_index: usize) {
    let BezierPairCaptureFailure::Curve { curve_index, cause } = failure else {panic!("expected captured curve context, got {failure:?}");};
    assert_eq!(*curve_index, expected_index);
    assert!(matches!(cause.as_ref(), BezierPairCaptureFailure::WeightCount {controls: 2,weights: 1}));
}
#[test]
fn actual_right_source_singularity_retains_domain_and_original_denominator_roots() {
    let controls = [[0., 0.], [0.5, 0.], [1., 1.]];
    let singular = BezierPairCurve {controls: &controls, weights: Some(&[1., 0., -1.]), interval: [0., 1.]};
    let query = CapturedBezierPair::capture_pair(curve(&controls), singular).unwrap();
    let report = query.certify(policy(), || false);
    assert_eq!(report.completion(), BezierPairCompletion::Incomplete);
    let Some(BezierPairFailure::Operation(failure)) = report.failure() else {panic!("expected original source failure");};
    assert!(matches!(failure, BezierPairOperationFailure::SourceDomain {source_index: 1, ..}));
    let BezierPairOperationFailure::DenominatorRoots(roots) = failure.original_cause() else {panic!("expected actual denominator root isolation");};
    assert_eq!(roots.roots()[0].bounds().0.compare_f64(0.5).unwrap(), std::cmp::Ordering::Equal);
    assert!(report.complete().is_none());
}
#[test]
fn actual_left_nonfinite_coordinate_is_invalid_and_preserves_original_point_cause() {
    let left = [[f64::NAN, 0.], [1., 1.]];
    let right = [[0., 1.], [1., 0.]];
    let query = CapturedBezierPair::capture_pair(curve(&left), curve(&right)).unwrap();
    let report = query.certify(policy(), || false);
    assert_eq!(report.completion(), BezierPairCompletion::Invalid);
    let Some(BezierPairFailure::Operation(failure)) = report.failure() else {panic!("expected original source failure");};
    assert!(matches!(failure, BezierPairOperationFailure::SourceDomain {source_index: 0, ..}));
    assert!(matches!(failure.original_cause(), BezierPairOperationFailure::Source(BezierPairCaptureFailure::Coordinate(_))));
    assert!(report.source().captured_curves()[0].controls[0][0].is_nan());
}

fn assert_capture_message(failure: &BezierPairCaptureFailure) {
    let message = failure.to_string();
    assert!(message.contains("controls: 2") && message.contains("weights: 1"));
    assert!(message.contains("one weight per control"));
}
