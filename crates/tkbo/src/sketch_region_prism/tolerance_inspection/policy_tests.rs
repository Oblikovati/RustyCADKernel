// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[test]
fn explicit_cap_modes_keep_world_budget_and_actual_values_separate() {
    for (policy, decision) in cap_expectations() {
        let request = PrismToleranceRequest::new(policy, 0.).unwrap();
        let observed = request.observe(1. / 8., 1. / 8.).unwrap();
        assert_eq!(
            observed,
            StoredToleranceObservation {
                raw: 1. / 8.,
                effective: 1. / 8.,
                policy,
                decision,
            }
        );
        assert_eq!(request.world_spatial_budget(), 0.);
    }
}

fn cap_expectations() -> [(StoredToleranceCap, StoredCapDecision); 3] {
    [
        (
            StoredToleranceCap::report(1. / 16.).unwrap(),
            StoredCapDecision::Reported,
        ),
        (
            StoredToleranceCap::refuse(1. / 16.).unwrap(),
            StoredCapDecision::Refused,
        ),
        (
            StoredToleranceCap::NoLimit,
            StoredCapDecision::ExplicitNoLimit,
        ),
    ]
}

#[test]
fn finite_cap_boundary_and_accessor_floor_do_not_widen_raw_metadata() {
    let policy = StoredToleranceCap::refuse(0.125).unwrap();
    let request = PrismToleranceRequest::new(policy, 1e-8).unwrap();
    assert_eq!(
        request.observe(0.125, 0.125).unwrap().decision,
        StoredCapDecision::Within
    );
    let floor = request.observe(0., 1e-7).unwrap();
    assert_eq!((floor.raw, floor.effective), (0., 1e-7));
    assert_eq!(request.world_spatial_budget(), 1e-8);
    let zero_cap = PrismToleranceRequest::new(StoredToleranceCap::refuse(0.).unwrap(), 0.).unwrap();
    assert_eq!(
        zero_cap.observe(0., 1e-7).unwrap().decision,
        StoredCapDecision::Within
    );
}

#[test]
fn invalid_raw_metadata_is_visible_even_with_finite_effective_value_and_no_limit() {
    let request = PrismToleranceRequest::new(StoredToleranceCap::NoLimit, 0.).unwrap();
    for invalid in [-1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = request.observe(invalid, 1e-7).unwrap_err();
        assert_eq!(error.quantity, ToleranceQuantity::RawStoredMetadata);
        assert_eq!(error.actual.to_bits(), invalid.to_bits());
    }
}

#[test]
fn finite_modes_and_world_budget_reject_nonfinite_or_negative_requests() {
    for invalid in [-1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for result in [
            StoredToleranceCap::report(invalid),
            StoredToleranceCap::refuse(invalid),
        ] {
            let error = result.unwrap_err();
            assert_eq!(error.quantity, ToleranceQuantity::StoredMetadataCap);
            assert_eq!(error.actual.to_bits(), invalid.to_bits());
        }
        let error = PrismToleranceRequest::new(StoredToleranceCap::NoLimit, invalid).unwrap_err();
        assert_eq!(error.quantity, ToleranceQuantity::WorldSpatialBudget);
        assert_eq!(error.actual.to_bits(), invalid.to_bits());
    }
}

#[test]
fn invalid_effective_accessor_observation_preserves_the_offending_scalar() {
    let request = PrismToleranceRequest::new(StoredToleranceCap::NoLimit, 0.).unwrap();
    let error = request.observe(0., f64::NAN).unwrap_err();
    assert_eq!(error.quantity, ToleranceQuantity::EffectiveAccessor);
    assert!(error.actual.is_nan());
    assert_eq!(
        error.to_string(),
        "EffectiveAccessor NaN; expected finite nonnegative scalar"
    );
}

#[test]
fn scalar_policy_and_observations_are_send_sync() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<PrismToleranceRequest>();
    send_sync::<StoredToleranceObservation>();
    send_sync::<ToleranceScalarError>();
}
