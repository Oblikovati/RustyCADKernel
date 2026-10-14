// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{RootIsolationError, certificate_arithmetic::ExactRatio};
use super::super::{MetricBezier, MetricCurve};
use super::*;
use PolynomialIntegralFailure as Failure;
use std::error::Error;

const CUBIC: [[f64; 2]; 4] = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
const POINT: [[f64; 2]; 1] = [[3., 4.]];
const FIXTURE: &str = include_str!("../polynomial-integrals-fixture.txt");
fn polynomial(poles: &[[f64; 2]]) -> OrientedMetricCurve<'_> {
    OrientedMetricCurve {
        curve: MetricCurve::Bezier(MetricBezier {
            poles,
            weights: None,
        }),
        reversed: false,
    }
}
fn line(origin: [f64; 2], direction: [f64; 2]) -> OrientedMetricCurve<'static> {
    OrientedMetricCurve {
        curve: MetricCurve::Line { origin, direction },
        reversed: false,
    }
}
fn frame() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
fn placement() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
fn occurrences(case: &str) -> Vec<OrientedMetricCurve<'static>> {
    match case {
        "cubic_ccw" | "cubic_reflected" => vec![polynomial(&CUBIC)],
        "cubic_cw" => vec![OrientedMetricCurve {
            reversed: true,
            ..polynomial(&CUBIC)
        }],
        "square" => square_occurrences(),
        "triangle" => vec![
            line([0., 0.], [1., 0.]),
            line([1., 0.], [-1., 1.]),
            line([0., 1.], [0., -1.]),
        ],
        "point" => vec![polynomial(&POINT)],
        "open_line" => vec![line([1., 2.], [2., 3.])],
        other => panic!("fixture case={other}; expected an owned fixture name"),
    }
}
fn square_occurrences() -> Vec<OrientedMetricCurve<'static>> {
    vec![
        line([0., 0.], [1., 0.]),
        line([1., 0.], [0., 1.]),
        line([1., 1.], [-1., 0.]),
        line([0., 1.], [0., -1.]),
    ]
}
fn rational(numerator: &str, denominator: &str) -> ExactRatio {
    ExactRatio::new(numerator.parse().unwrap(), denominator.parse().unwrap())
}
fn measure(curves: &[OrientedMetricCurve<'_>]) -> CertifiedPolynomialMeasure {
    certify_polynomial_boundary_integral(curves, PolynomialIntegralOptions::default(), || false)
        .unwrap()
}
fn assert_exact_enclosure(report: &CertifiedPolynomialMeasure, expected: &ExactRatio) {
    let [lower, upper] = report.enclosure().map(ExactRatio::from_finite);
    assert!(lower <= *expected && *expected <= upper);
    let error = expected.subtract(&ExactRatio::from_finite(report.representative()));
    assert!(ExactRatio::from_finite(report.absolute_error_bound()) >= error);
    if report.absolute_error_bound() > 0. {
        let prior = f64::from_bits(report.absolute_error_bound().to_bits() - 1);
        assert!(ExactRatio::from_finite(prior) < error);
    }
    assert_eq!(report.requested_options(), report.effective_options());
    assert_eq!(
        report.evidence(),
        PolynomialIntegralEvidence::CertifiedBound
    );
}
#[test]
fn owned_independent_fixtures_pin_boundary_and_sweep_integrals() {
    for row in FIXTURE.lines().filter(|row| !row.starts_with('#')) {
        let fields: Vec<&str> = row.split_whitespace().collect();
        let curves = occurrences(fields[0]);
        assert_boundary_fixture(&curves, rational(fields[1], fields[2]));
        let vector = [0., 0., fields[3].parse().unwrap()];
        let report = sweep_measure(&curves, vector);
        assert_exact_enclosure(&report, &rational(fields[4], fields[5]));
        assert_eq!(
            report.operation(),
            PolynomialIntegralOperation::SignedSweepIntegral
        );
    }
}
fn sweep_measure(
    curves: &[OrientedMetricCurve<'_>],
    vector: [f64; 3],
) -> CertifiedPolynomialMeasure {
    certify_polynomial_sweep_integral(
        curves,
        frame(),
        placement(),
        vector,
        PolynomialIntegralOptions::default(),
        || false,
    )
    .unwrap()
}
fn assert_boundary_fixture(curves: &[OrientedMetricCurve<'_>], expected: ExactRatio) {
    let report = measure(curves);
    assert_exact_enclosure(&report, &expected);
    assert_eq!(
        report.operation(),
        PolynomialIntegralOperation::SignedBoundaryIntegral
    );
    let request = IntegralRequest {
        options: report.requested_options(),
        operation: report.operation(),
    };
    assert_eq!(request.integrate(curves, &mut || false).unwrap(), expected);
}
#[test]
fn all_public_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OrientedMetricCurve<'static>>();
    assert_send_sync::<PolynomialIntegralOptions>();
    assert_send_sync::<PolynomialIntegralOperation>();
    assert_send_sync::<PolynomialIntegralEvidence>();
    assert_send_sync::<CertifiedPolynomialMeasure>();
    assert_send_sync::<PolynomialIntegralError>();
    assert_send_sync::<PolynomialIntegralFailure>();
}
#[test]
fn error_budget_refuses_inexact_rounding_without_widening_or_changing_exact_values() {
    let options = PolynomialIntegralOptions {
        absolute_error_limit: 0.,
        ..Default::default()
    };
    let error = certify_polynomial_boundary_integral(&occurrences("cubic_ccw"), options, || false)
        .unwrap_err();
    assert!(
        matches!(error.cause, PolynomialIntegralFailure::ErrorLimit { requested: 0., achieved } if achieved > 0.)
    );
    let exact =
        certify_polynomial_boundary_integral(&occurrences("square"), options, || false).unwrap();
    assert_eq!(exact.enclosure(), [1., 1.]);
    assert_eq!(exact.absolute_error_bound(), 0.);
}
fn failure(curves: &[OrientedMetricCurve<'_>], options: PolynomialIntegralOptions) -> Failure {
    certify_polynomial_boundary_integral(curves, options, || false)
        .unwrap_err()
        .cause
}
#[test]
fn invalid_policy_is_explicit_without_clamps() {
    for (version, degree, curves) in [(2, 16, 64), (1, 17, 64), (1, 16, 65), (1, 16, 0)] {
        let options = PolynomialIntegralOptions {
            behavior_version: version,
            max_degree: degree,
            max_curves: curves,
            ..Default::default()
        };
        assert!(
            matches!(failure(&occurrences("point"), options), Failure::Policy(actual) if actual == options)
        );
    }
}
#[test]
fn invalid_error_cap_is_explicit_without_clamps() {
    for absolute_error_limit in [-1., f64::INFINITY, f64::NAN] {
        let options = PolynomialIntegralOptions {
            absolute_error_limit,
            ..Default::default()
        };
        assert!(matches!(
            failure(&occurrences("point"), options),
            Failure::Policy(_)
        ));
    }
}
#[test]
fn empty_and_excess_work_are_incomplete() {
    let options = PolynomialIntegralOptions::default();
    assert!(matches!(failure(&[], options), Failure::EmptyBoundary));
    let oversized = [polynomial(&POINT); 65];
    assert!(matches!(
        failure(&oversized, options),
        Failure::CurveCount {
            actual: 65,
            limit: 64
        }
    ));
    let poles = [[0.; 2]; 18];
    assert!(matches!(
        failure(&[polynomial(&poles)], options),
        Failure::Degree {
            actual: 17,
            limit: 16
        }
    ));
}
#[test]
fn rational_weights_are_incomplete_even_when_equal() {
    let rational = OrientedMetricCurve {
        curve: MetricCurve::Bezier(MetricBezier {
            poles: &CUBIC,
            weights: Some(&[1.; 4]),
        }),
        reversed: false,
    };
    let error = failure(&[rational], PolynomialIntegralOptions::default());
    assert!(matches!(error, Failure::RationalWeights { count: 4 }));
}
#[test]
fn metric_failure_retains_original_cause_and_occurrence_index() {
    let invalid = [polynomial(&POINT), line([f64::NAN, 0.], [0., 1.])];
    let error = certify_polynomial_boundary_integral(
        &invalid,
        PolynomialIntegralOptions::default(),
        || false,
    )
    .unwrap_err();
    assert_eq!(error.curve_index, Some(1));
    assert!(matches!(
        &error.cause,
        PolynomialIntegralFailure::Metric(cause) if matches!(cause.as_ref(), super::super::ResidualError::Value { role: "pole", .. })
    ));
    assert!(error.source().unwrap().source().is_some());
    assert!(error.to_string().contains("expected"));
    assert!(error.to_string().contains("occurrence Some(1)"));
}
#[test]
fn source_changes_and_clones_cannot_change_published_reports() {
    let mut poles = CUBIC;
    let report = measure(&[polynomial(&poles)]);
    let cloned = report.clone();
    poles.fill([0.; 2]);
    assert_eq!(measure(&[polynomial(&poles)]).enclosure(), [0., 0.]);
    assert_eq!(report, cloned);
    assert_exact_enclosure(&report, &rational("12", "5"));
}

#[path = "stress_tests.rs"]
mod stress_tests;

#[test]
fn failure_and_cancellation_preserve_supplied_controls() {
    let poles = CUBIC;
    let curves = [polynomial(&poles)];
    let options = PolynomialIntegralOptions {
        absolute_error_limit: 0.,
        ..Default::default()
    };
    assert!(certify_polynomial_boundary_integral(&curves, options, || false).is_err());
    assert!(certify_polynomial_boundary_integral(&curves, options, || true).is_err());
    assert_eq!(poles, CUBIC);
    assert_exact_enclosure(&measure(&curves), &rational("12", "5"));
}

#[test]
fn empty_control_curve_is_incomplete_with_its_occurrence_index() {
    let curves = [polynomial(&POINT), polynomial(&[])];
    let error =
        certify_polynomial_boundary_integral(&curves, Default::default(), || false).unwrap_err();
    assert_eq!(error.curve_index, Some(1));
    assert!(matches!(&error.cause, Failure::Metric(cause)
        if matches!(cause.as_ref(), super::super::ResidualError::Shape { poles:0, .. })));
}

#[test]
fn invalid_singular_sweep_inputs_retain_typed_operation_context() {
    let mut invalid = placement();
    invalid.origin[2] = f64::INFINITY;
    let curves = occurrences("point");
    let options = PolynomialIntegralOptions::default();
    let error =
        certify_polynomial_sweep_integral(&curves, frame(), invalid, [0.; 3], options, || false)
            .unwrap_err();
    assert_eq!(
        error.operation,
        PolynomialIntegralOperation::SignedSweepIntegral
    );
    assert_eq!(error.requested_options, options);
    assert_eq!(error.curve_index, None);
    assert!(matches!(error.cause, Failure::Metric(cause)
        if matches!(*cause, super::super::ResidualError::Value { role:"origin", index:2, .. })));
}

#[path = "plane/tests.rs"]
mod plane_tests;
