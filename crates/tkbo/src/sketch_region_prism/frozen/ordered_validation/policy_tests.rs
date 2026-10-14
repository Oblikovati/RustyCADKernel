// SPDX-License-Identifier: GPL-2.0-only
use super::{test_support::*, *};
use crate::sketch_region_prism::tolerance_inspection::StoredCapDecision;
#[test]
fn report_exceedance_completes_with_diagnostic_and_explicit_policy_changes_bytes() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let request = OrderedPrismValidationOptions::v1(StoredToleranceCap::report(0.).unwrap());
    let outcome = validate_joined_prism(&join, request, &mut NamedValidationProgress::default());
    let OrderedPrismValidationOutcome::CompleteValidation(proof) = outcome else {
        panic!("expected completed report-mode diagnostic")
    };
    assert!(
        proof
            .tolerance()
            .metadata()
            .iter()
            .any(|row| row.observation.decision == StoredCapDecision::Reported)
    );
    assert_eq!(proof.requested(), request);
    assert_ne!(proof.replay_bytes(), complete(&join).replay_bytes());
}
#[test]
fn unlimited_metadata_policy_requires_explicit_selection_and_is_retained() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let request = OrderedPrismValidationOptions::v1(StoredToleranceCap::NoLimit);
    let outcome = validate_joined_prism(&join, request, &mut NamedValidationProgress::default());
    let OrderedPrismValidationOutcome::CompleteValidation(proof) = outcome else {
        panic!("expected explicit unlimited metadata policy")
    };
    assert_eq!(proof.tolerance().cap(), StoredToleranceCap::NoLimit);
    assert!(
        proof
            .tolerance()
            .metadata()
            .iter()
            .all(|row| row.observation.decision == StoredCapDecision::ExplicitNoLimit)
    );
    assert_ne!(proof.replay_bytes(), complete(&join).replay_bytes());
}
