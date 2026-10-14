// SPDX-License-Identifier: GPL-2.0-only
//! Exact constant equations prove whole-domain emptiness without graph substitutes.
use rustycad_tkmath::predicates::roots::{
    RootIsolationError, RootIsolationOptions, bezier_pair::*,
};
fn policy() -> BezierPairPolicy {
    let mut roots = RootIsolationOptions::new(1e-9);
    roots.max_bisections = 0;
    BezierPairPolicy {
        version: 1,
        roots,
        signs: BezierPairSignOptions::derived(),
        requested_world_error: 1e-8,
    }
}
fn curve(controls: &[[f64; 2]]) -> BezierPairCurve<'_> {
    BezierPairCurve {
        controls,
        weights: None,
        interval: [-1., 2.],
    }
}
fn self_line(controls: &[[f64; 2]]) -> CapturedBezierPair {
    CapturedBezierPair::self_off_diagonal(controls, None, [-2., 3.]).unwrap()
}
fn assert_excluded(report: &BezierPairReport, axis: usize, value: i64, self_domain: bool) {
    assert_eq!(report.completion(), BezierPairCompletion::Complete);
    assert!(report.failure().is_none());
    let certificate = report.complete().unwrap();
    assert_eq!(certificate.contact_count(), 0);
    assert!(certificate.contact(0).is_none() && certificate.graph().is_none());
    assert_skipped_graph(&certificate);
    assert_witness(
        &certificate.constant_exclusion().unwrap(),
        axis,
        value,
        self_domain,
    );
    assert_empty_domain_work(report);
}
fn assert_empty_domain_work(report: &BezierPairReport) {
    let certificate = report.complete().unwrap();
    assert_eq!(certificate.work(), report.work());
    assert_eq!(report.work().completed, 0);
    assert!(
        certificate
            .denominators()
            .iter()
            .all(|proof| proof.roots().is_empty())
    );
}
fn assert_skipped_graph(certificate: &BezierPairCertificate<'_>) {
    assert_eq!(
        certificate.sign_stage(),
        BezierPairSignStage::SkippedConstantExclusion
    );
}
fn assert_witness(
    witness: &BezierPairConstantExclusion<'_>,
    axis: usize,
    value: i64,
    self_domain: bool,
) {
    assert_eq!(witness.coordinate_axis(), axis);
    assert_eq!(witness.constant().polynomial_degree(), Some(0));
    assert_eq!(constant_value(witness), value.to_string());
    assert_eq!(witness.divided_self_diagonal(), self_domain);
    assert!(witness.equation(2).is_none());
    assert_eq!(witness.equation(axis).unwrap().len(), 1);
}
fn constant_value(witness: &BezierPairConstantExclusion<'_>) -> String {
    let polynomial = witness.constant();
    let coefficient = polynomial.signed_coefficient(0).unwrap();
    coefficient.to_string()
}
#[test]
fn exact_linear_self_quotient_and_reversal_cover_both_coordinate_axes() {
    for (controls, axis, constant) in [
        ([[0., 0.], [1., 0.]], 0, 1),
        ([[1., 0.], [0., 0.]], 0, -1),
        ([[0., 0.], [0., 2.]], 1, 2),
    ] {
        let report = self_line(&controls).certify(policy(), || false);
        assert_excluded(&report, axis, constant, true);
        assert_eq!(report.source().captured_curves()[0].interval, [-2., 3.]);
    }
}
#[test]
fn arbitrary_degree_separation_uses_constant_equation_not_curve_recognition() {
    for degree in 2..=7 {
        let a: Vec<[f64; 2]> = (0..=degree)
            .map(|i| [0., (i as i64).pow(degree as u32) as f64])
            .collect();
        let b: Vec<[f64; 2]> = (0..=degree)
            .map(|i| [7., (i as i64).pow(degree as u32) as f64 + 3.])
            .collect();
        let query = CapturedBezierPair::capture_pair(curve(&a), curve(&b)).unwrap();
        let report = query.certify(policy(), || false);
        assert_excluded(&report, 0, -7, false);
        assert_eq!(
            report.complete().unwrap().captured_curves()[0]
                .controls
                .len(),
            degree + 1
        );
    }
}
#[test]
fn source_mutation_and_negative_constant_weights_do_not_detach_proof() {
    let mut controls = [[0., 0.], [1., 0.]];
    let mut weights = [-2., -2.];
    let query = CapturedBezierPair::self_off_diagonal(&controls, Some(&weights), [0., 1.]).unwrap();
    controls.fill([f64::NAN, f64::NAN]);
    weights.fill(0.);
    let report = query.certify(policy(), || false);
    drop(query);
    assert_excluded(&report, 0, 1, true);
    assert_eq!(
        report.source().captured_curves()[0].weights.unwrap(),
        &[-2., -2.]
    );
    assert_negative_denominator(&report);
}
fn assert_negative_denominator(report: &BezierPairReport) {
    assert_eq!(
        report.complete().unwrap().homogeneous_sources()[0][2]
            .signed_coefficient(0)
            .unwrap()
            .to_string(),
        "-1"
    );
}
#[test]
fn final_projection_poll_cancellation_preserves_cause_and_successful_retry() {
    let query = self_line(&[[0., 0.], [1., 0.]]);
    let mut polls = 0;
    let success = query.certify(policy(), || {
        polls += 1;
        false
    });
    assert_excluded(&success, 0, 1, true);
    let mut retry_polls = 0;
    let cancelled = query.certify(policy(), || {
        retry_polls += 1;
        retry_polls == polls
    });
    assert_eq!(cancelled.completion(), BezierPairCompletion::Cancelled);
    assert!(cancelled.complete().is_none());
    assert_root_cancel(&cancelled);
    assert_eq!(cancelled.work().completed, 0);
    assert_excluded(&query.certify(policy(), || false), 0, 1, true);
}
fn assert_root_cancel(report: &BezierPairReport) {
    let BezierPairFailure::Operation(cause) = report.failure().unwrap() else {
        panic!("expected operation cause")
    };
    assert!(matches!(
        cause,
        BezierPairOperationFailure::Root(RootIsolationError::Cancelled)
    ));
    assert_eq!(
        report.source().captured_curves()[0].controls,
        &[[0., 0.], [1., 0.]]
    );
}
#[test]
fn zero_and_nonlinear_self_domains_remain_incomplete_without_forged_certificates() {
    for controls in [
        &[[0., 0.], [0., 0.]][..],
        &[[0., 0.], [0., 0.], [1., 0.], [1., 0.]][..],
    ] {
        let query = self_line(controls);
        let report = query.certify(policy(), || false);
        assert_eq!(report.completion(), BezierPairCompletion::Incomplete);
        assert!(report.complete().is_none());
        assert!(report.failure().is_some());
    }
}
#[test]
fn requested_admission_is_not_silently_widened_by_constant_exclusion() {
    let query = self_line(&[[0., 0.], [1., 0.]]);
    let mut options = policy();
    options.roots.max_degree = 0;
    options.signs.degree_limit = BezierPairSignAdmission::Explicit(0);
    let report = query.certify(options, || false);
    assert_excluded(&report, 0, 1, true);
    assert_eq!(report.policy(), options);
    assert_eq!(
        report.complete().unwrap().sign_stage(),
        BezierPairSignStage::SkippedConstantExclusion
    );
}
#[test]
fn invalid_options_preserve_order_and_repair_reuses_the_owned_query() {
    let query = self_line(&[[0., 0.], [1., 0.]]);
    let mut options = policy();
    options.roots.version = 99;
    options.signs.version = 77;
    let report = query.certify(options, || false);
    assert_eq!(report.completion(), BezierPairCompletion::Invalid);
    assert!(matches!(
        operation_cause(&report),
        BezierPairOperationFailure::Root(RootIsolationError::OptionsVersion(99))
    ));
    options.roots.version = 1;
    let report = query.certify(options, || false);
    assert_eq!(report.completion(), BezierPairCompletion::Invalid);
    assert!(matches!(
        operation_cause(&report),
        BezierPairOperationFailure::SignPolicy(BezierPairSignFailure::Version(77))
    ));
    assert_excluded(&query.certify(policy(), || false), 0, 1, true);
}
fn operation_cause(report: &BezierPairReport) -> &BezierPairOperationFailure {
    let Some(BezierPairFailure::Operation(cause)) = report.failure() else {
        panic!("expected typed operation failure")
    };
    assert!(report.complete().is_none());
    cause.original_cause()
}
#[test]
fn invalid_world_error_precedes_cancellation_and_no_geometry_is_exposed() {
    let query = self_line(&[[0., 0.], [1., 0.]]);
    let mut options = policy();
    options.requested_world_error = -3.;
    let mut polls = 0;
    let report = query.certify(options, || {
        polls += 1;
        true
    });
    assert_eq!(polls, 0);
    assert_eq!(report.completion(), BezierPairCompletion::Invalid);
    assert!(matches!(
        operation_cause(&report),
        BezierPairOperationFailure::WorldRequest(-3.)
    ));
    assert_excluded(&query.certify(policy(), || false), 0, 1, true);
}
#[test]
fn singular_denominator_cannot_be_bypassed_by_an_empty_equation_argument() {
    let query =
        CapturedBezierPair::self_off_diagonal(&[[0., 0.], [1., 0.]], Some(&[1., -1.]), [0., 1.])
            .unwrap();
    let exhausted = query.certify(policy(), || false);
    assert_eq!(exhausted.completion(), BezierPairCompletion::Incomplete);
    assert!(matches!(
        operation_cause(&exhausted),
        BezierPairOperationFailure::Root(RootIsolationError::SubdivisionLimit { limit: 0 })
    ));
    assert_eq!(exhausted.work().completed, 0);
    assert_source_failure_index(&exhausted, 0);
    let mut options = policy();
    options.roots.max_bisections = 1;
    let singular = query.certify(options, || false);
    assert_denominator_pole(&singular);
}
fn assert_denominator_pole(report: &BezierPairReport) {
    assert_eq!(report.completion(), BezierPairCompletion::Incomplete);
    let BezierPairOperationFailure::DenominatorRoots(roots) = operation_cause(report) else {
        panic!(
            "expected actual denominator roots; cause {:?}",
            report.failure()
        )
    };
    assert_eq!(roots.roots().len(), 1);
    assert_eq!(
        roots.roots()[0].bounds().0.compare_f64(0.5).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(report.work().completed, 1);
    assert_eq!(roots.bisections(), 1);
    assert_source_failure_index(report, 0);
}
fn assert_source_failure_index(report: &BezierPairReport, expected: usize) {
    let Some(BezierPairFailure::Operation(BezierPairOperationFailure::SourceDomain {
        source_index,
        ..
    })) = report.failure()
    else {
        panic!("expected retained source-domain context")
    };
    assert_eq!(*source_index, expected);
    assert_eq!(
        report.source().captured_curves()[expected].interval,
        [0., 1.]
    );
}
#[test]
fn degree_elevated_self_lines_retain_exact_constant_not_a_shape_tag() {
    for degree in 1..=7 {
        let controls: Vec<[f64; 2]> = (0..=degree).map(|index| [index as f64, 0.]).collect();
        let query = self_line(&controls);
        let report = query.certify(policy(), || false);
        assert_excluded(&report, 0, degree as i64, true);
        assert_eq!(report.source().captured_curves()[0].controls, controls);
    }
}
#[test]
fn coordinate_swap_and_source_permutation_preserve_exact_exclusion() {
    let a = [[0., 0.], [0., 1.], [0., 3.]];
    let b = [[7., 3.], [7., 4.], [7., 6.]];
    for (left, right, axis, constant) in [
        (a, b, 0, -7),
        (b, a, 0, 7),
        (a.map(|p| [p[1], p[0]]), b.map(|p| [p[1], p[0]]), 1, -7),
    ] {
        let query = CapturedBezierPair::capture_pair(curve(&left), curve(&right)).unwrap();
        assert_excluded(&query.certify(policy(), || false), axis, constant, false);
    }
}
fn rational_linear_self_query() -> CapturedBezierPair {
    let controls = [[0., 0.], [0.5, 0.]];
    let weights = [1., 2.];
    CapturedBezierPair::self_off_diagonal(&controls, Some(&weights), [0., 1.]).unwrap()
}
#[test]
fn nonconstant_regular_denominator_remains_bound_to_rational_exclusion() {
    // The captured source is x=t/(1+t); its exact diagonal quotient is one.
    let report = rational_linear_self_query().certify(policy(), || false);
    assert_excluded(&report, 0, 1, true);
    let denominator = report.complete().unwrap().homogeneous_sources()[0][2];
    assert_eq!(denominator.polynomial_degree(), Some(1));
    assert_eq!(denominator.signed_coefficient(0).unwrap().to_string(), "1");
    assert_eq!(denominator.signed_coefficient(1).unwrap().to_string(), "1");
}
#[test]
fn constant_witness_does_not_bypass_denominator_degree_admission() {
    let query = rational_linear_self_query();
    let mut options = policy();
    options.roots.max_degree = 0;
    let rejected = query.certify(options, || false);
    assert_eq!(rejected.completion(), BezierPairCompletion::Incomplete);
    assert!(matches!(
        operation_cause(&rejected),
        BezierPairOperationFailure::Root(RootIsolationError::Degree {
            actual: 1,
            limit: 0
        })
    ));
    assert_eq!(rejected.work().completed, 0);
    assert_source_failure_index(&rejected, 0);
    assert_excluded(&query.certify(policy(), || false), 0, 1, true);
}
