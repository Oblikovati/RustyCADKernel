// SPDX-License-Identifier: GPL-2.0-only
use super::{joined_test_support::*, *};
use crate::sketch_region_prism::{frozen::FrozenPrismNode, tolerance_inspection::*};
fn node(join: &JoinedPrismProof) -> FrozenPrismNode {
    join.projection()
        .nodes()
        .iter()
        .find(|row| row.key.kind == 7)
        .unwrap()
        .clone()
}
fn observe(
    join: &JoinedPrismProof,
    node: &FrozenPrismNode,
    cap: StoredToleranceCap,
) -> Result<JoinedToleranceMetadata, JoinedToleranceCause> {
    super::stored_metadata::observe_node(
        join.projection(),
        node,
        PrismToleranceRequest::new(cap, 0.125).unwrap(),
    )
}
#[test]
fn owned_zero_subfloor_metadata_and_invalid_raw_values_keep_actual_bits() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut raw = node(&join);
    for value in [0., -0., f64::MIN_POSITIVE, 0.125] {
        raw.raw_tolerance = Some(value);
        let row = observe(&join, &raw, StoredToleranceCap::NoLimit)
            .ok()
            .unwrap();
        assert_eq!(row.observation.raw.to_bits(), value.to_bits());
        assert_eq!(
            row.observation.effective.to_bits(),
            raw.effective_tolerance.unwrap().to_bits()
        );
    }
    invalid_metadata(&join, &mut raw);
}
fn invalid_metadata(join: &JoinedPrismProof, raw: &mut FrozenPrismNode) {
    for value in [-1., f64::INFINITY, f64::from_bits(0x7ff8000000000123)] {
        raw.raw_tolerance = Some(value);
        let failure = observe(join, raw, StoredToleranceCap::NoLimit)
            .err()
            .unwrap();
        let JoinedToleranceCause::MetadataScalar { entity, cause } = failure else {
            panic!("expected typed raw scalar")
        };
        assert_eq!(entity, raw.key);
        assert_eq!(cause.actual.to_bits(), value.to_bits());
        assert_eq!(cause.quantity, ToleranceQuantity::RawStoredMetadata);
    }
}
#[test]
fn missing_effective_and_raw_metadata_are_incomplete_with_entity_context() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut raw = node(&join);
    raw.raw_tolerance = None;
    assert!(matches!(
        observe(&join, &raw, StoredToleranceCap::NoLimit),
        Err(JoinedToleranceCause::MissingMetadata { .. })
    ));
    raw.raw_tolerance = Some(0.);
    raw.effective_tolerance = None;
    assert!(matches!(
        observe(&join, &raw, StoredToleranceCap::NoLimit),
        Err(JoinedToleranceCause::MissingMetadata { .. })
    ));
}
#[test]
fn raw_one_eighth_cap_one_sixteenth_preserves_report_and_refuse_decisions() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut raw = node(&join);
    raw.raw_tolerance = Some(0.125);
    let report = observe(&join, &raw, StoredToleranceCap::report(0.0625).unwrap())
        .ok()
        .unwrap();
    assert_eq!(report.observation.decision, StoredCapDecision::Reported);
    let refused = observe(&join, &raw, StoredToleranceCap::refuse(0.0625).unwrap())
        .ok()
        .unwrap();
    assert_eq!(refused.observation.decision, StoredCapDecision::Refused);
    assert!(super::stored_metadata::refuse_caps(&[refused]).is_err());
    assert!(super::stored_metadata::refuse_caps(&[report]).is_ok());
}
