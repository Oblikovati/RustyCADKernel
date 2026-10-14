// SPDX-License-Identifier: GPL-2.0-only
//! Exact native-unit error evidence remains separate from contact identity and root brackets.
use super::super::metric_bounds::MetricParameterMap;
use super::exact_parameter_retention_tests::retained_rational_contact;
use super::*;
use std::cmp::Ordering;

#[test]
fn non_singleton_root_enclosure_cannot_prove_zero_construction_error() {
    let retained = super::algebraic_world_tests::irrational_proof();
    let proof = BezierPairCertificate::new(&retained);
    let contact = proof.contact(0).unwrap();
    let native = [contact.point[0]; 2];
    let measured = proof
        .measure_native_parameters(0, identity_maps(), native, [0.; 2])
        .unwrap()
        .unwrap();
    assert_eq!(measured[0].decision(), PairParameterDecision::Inconclusive);
    assert!(
        measured[0]
            .error_enclosure()
            .compare_upper(0.)
            .unwrap()
            .is_gt()
    );
    assert!(contact.parameter.width().compare_f64(0.).unwrap().is_gt());
}

#[test]
fn rational_third_native_display_has_nonzero_exact_error_under_zero_limit() {
    let report = retained_rational_contact();
    let proof = report.complete().unwrap();
    let measured = parameter_measurements(&report, identity_maps(), [0.5, 1. / 3.], [0., 0.]);
    assert_eq!(measured[0].decision(), PairParameterDecision::Within);
    assert_eq!(measured[1].decision(), PairParameterDecision::Exceeds);
    assert_eq!(
        measured[1].error_enclosure().compare_lower(0.).unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        measured[1].native_parameter().to_bits(),
        (1_f64 / 3.).to_bits()
    );
    assert_original_half_root(&proof);
}

#[test]
fn requested_limit_is_preserved_without_erasing_measured_rational_error() {
    let report = retained_rational_contact();
    let measured = parameter_measurements(&report, identity_maps(), [0.5, 1. / 3.], [0., 1e-8]);
    assert_eq!(measured[1].decision(), PairParameterDecision::Within);
    assert_eq!(measured[1].requested_limit(), 1e-8);
    assert_eq!(
        measured[1].error_enclosure().compare_lower(0.).unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        measured[1].error_enclosure().compare_upper(1e-8).unwrap(),
        Ordering::Less
    );
}

#[test]
fn native_affine_transport_forms_unrounded_endpoint_difference_and_handles_reversal() {
    let report = retained_rational_contact();
    for endpoints in [[2., 5.], [5., 2.]] {
        let maps = [
            MetricParameterMap::Identity,
            MetricParameterMap::Endpoints(endpoints),
        ];
        let native = if endpoints[0] == 2. { 3. } else { 4. };
        let measured = parameter_measurements(&report, maps, [0.5, native], [0., 0.]);
        assert_affine_zero(&measured[1], native);
    }
}

#[test]
fn invalid_native_values_limits_and_maps_return_typed_original_values() {
    let report = retained_rational_contact();
    let proof = report.complete().unwrap();
    for (maps, native, limits) in invalid_cases() {
        assert!(
            proof
                .measure_native_parameters(0, maps, native, limits)
                .is_err()
        );
    }
    assert!(
        proof
            .measure_native_parameters(1, identity_maps(), [0., 0.], [0., 0.])
            .unwrap()
            .is_none()
    );
    assert_eq!(proof.contact_count(), 1);
}

#[test]
fn exact_affine_extremes_do_not_round_span_to_infinity_or_error_to_zero() {
    let report = retained_rational_contact();
    let maps = [
        MetricParameterMap::Identity,
        MetricParameterMap::Endpoints([f64::MAX, -f64::MAX]),
    ];
    let measured = parameter_measurements(&report, maps, [0.5, 0.], [0., f64::from_bits(1)]);
    assert_eq!(measured[1].decision(), PairParameterDecision::Exceeds);
    assert_eq!(
        measured[1].native_enclosure().compare_lower(0.).unwrap(),
        Ordering::Greater
    );
    assert_finite_extreme(&measured[1]);
}

fn identity_maps() -> [MetricParameterMap; 2] {
    let identity = MetricParameterMap::Identity;
    [identity, identity]
}

fn parameter_measurements(
    report: &BezierPairReport,
    maps: [MetricParameterMap; 2],
    native: [f64; 2],
    limits: [f64; 2],
) -> [MeasuredPairParameter; 2] {
    let proof = report.complete().unwrap();
    proof
        .measure_native_parameters(0, maps, native, limits)
        .unwrap()
        .unwrap()
}

fn assert_original_half_root(proof: &BezierPairCertificate<'_>) {
    assert_eq!(
        proof
            .contact(0)
            .unwrap()
            .parameter
            .bounds()
            .0
            .compare_f64(0.5)
            .unwrap(),
        Ordering::Equal
    );
}

fn assert_affine_zero(measured: &MeasuredPairParameter, native: f64) {
    assert_eq!(measured.decision(), PairParameterDecision::Within);
    assert_eq!(
        measured.native_enclosure().compare_lower(native).unwrap(),
        Ordering::Equal
    );
    assert_eq!(
        measured.error_enclosure().compare_upper(0.).unwrap(),
        Ordering::Equal
    );
}

type InvalidParameterCase = ([MetricParameterMap; 2], [f64; 2], [f64; 2]);
fn invalid_cases() -> [InvalidParameterCase; 3] {
    let invalid_map = [MetricParameterMap::Endpoints([0., f64::INFINITY]); 2];
    [
        (identity_maps(), [f64::NAN, 0.], [0., 0.]),
        (identity_maps(), [0., 0.], [-1., 0.]),
        (invalid_map, [0., 0.], [0., 0.]),
    ]
}

fn assert_finite_extreme(measured: &MeasuredPairParameter) {
    assert_eq!(
        measured.native_enclosure().compare_upper(f64::MAX).unwrap(),
        Ordering::Less
    );
}

#[test]
fn parameter_failure_preserves_canonical_axis_and_original_error_chain() {
    use std::error::Error;
    let report = retained_rational_contact();
    let proof = report.complete().unwrap();
    let failed =
        proof.measure_native_parameters(0, identity_maps(), [0.5, f64::INFINITY], [0., 0.]);
    let Err(cause) = failed else {
        panic!("infinite native u; expected typed parameter failure");
    };
    assert_eq!(cause.axis, 1);
    assert!(
        matches!(cause.cause, super::super::metric_bounds::ResidualError::Value { index: 1, value, .. } if value == f64::INFINITY)
    );
    assert!(cause.to_string().contains("axis=1"));
    assert!(cause.source().is_some());
}

#[test]
fn construction_measurements_and_failures_are_send_and_sync() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<MeasuredPairParameter>();
    require_send_sync::<PairParameterDecision>();
    require_send_sync::<PairParameterConstructionFailure>();
}
