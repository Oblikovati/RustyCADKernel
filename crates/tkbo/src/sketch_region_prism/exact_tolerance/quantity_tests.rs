// SPDX-License-Identifier: GPL-2.0-only
//! Metadata allowance, operation budget and exact query remain independent quantities.
use super::super::tolerance_inspection::{
    PrismToleranceRequest, StoredCapDecision, StoredToleranceCap, ToleranceQuantity,
};
use super::{scalar::*, test_support::*};
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

#[test]
fn raw_zero_and_subfloor_metadata_do_not_replace_exact_geometry() {
    let request =
        PrismToleranceRequest::new(StoredToleranceCap::refuse(1.).unwrap(), 0.125).unwrap();
    for raw in [0., -0., f64::from_bits(1), 0.000_000_05] {
        let observation = request.observe(raw, 0.000_000_1).unwrap();
        assert_eq!(observation.raw.to_bits(), raw.to_bits());
        assert_eq!(observation.effective, 0.000_000_1);
        assert_eq!(observation.decision, StoredCapDecision::Within);
        assert_zero_at_independent_budget(request);
    }
}

#[test]
fn reported_refused_and_explicit_unlimited_caps_retain_metadata_decisions() {
    for (cap, expected) in cap_cases() {
        let request = PrismToleranceRequest::new(cap, 0.125).unwrap();
        let observation = request.observe(0.125, 0.125).unwrap();
        assert_eq!(observation.decision, expected);
        assert_eq!((observation.raw, observation.effective), (0.125, 0.125));
        assert_zero_at_independent_budget(request);
    }
}

fn cap_cases() -> [(StoredToleranceCap, StoredCapDecision); 3] {
    [
        (
            StoredToleranceCap::report(0.0625).unwrap(),
            StoredCapDecision::Reported,
        ),
        (
            StoredToleranceCap::refuse(0.0625).unwrap(),
            StoredCapDecision::Refused,
        ),
        (
            StoredToleranceCap::NoLimit,
            StoredCapDecision::ExplicitNoLimit,
        ),
    ]
}

#[test]
fn invalid_raw_metadata_is_not_hidden_by_an_accessor_floor_or_no_limit() {
    let request = PrismToleranceRequest::new(StoredToleranceCap::NoLimit, 0.125).unwrap();
    for actual in [-1., f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let cause = request.observe(actual, 0.000_000_1).unwrap_err();
        assert_eq!(cause.quantity, ToleranceQuantity::RawStoredMetadata);
        assert_eq!(cause.actual.to_bits(), actual.to_bits());
    }
}

fn assert_zero_at_independent_budget(request: PrismToleranceRequest) {
    let certificate = certify_exact_endpoint(&constant_curve([0.; 3]), 0., [0.; 3], identity())
        .ok()
        .unwrap();
    assert_eq!(request.world_spatial_budget().to_bits(), 0.125f64.to_bits());
    assert_eq!(
        (certificate.requested_spatial, certificate.effective_spatial),
        (0., 0.)
    );
    let changed = certify_exact_endpoint(&constant_curve([0.; 3]), 0., [0.125, 0., 0.], identity());
    assert!(
        matches!(changed, Err(ExactEndpointFailure::Residual(report))
        if matches!(report.decision, Ok(ResidualDecision::Exceeds { .. })))
    );
}
