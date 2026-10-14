// SPDX-License-Identifier: GPL-2.0-only
//! Explicit missing coverage is part of the receipt and cannot be confused with a completed projection.
use super::{super::*, cause, gap, tags};
#[test]
fn diagnostic_classes_use_explicit_stable_tags() {
    use PrismDiagnosticClass::*;
    for (class, expected) in [(Declined, 0), (Incomplete, 1), (Failed, 2), (Cancelled, 3)] {
        assert_eq!(tags::class(class), expected);
    }
}
#[test]
fn local_classes_keep_the_same_vocabulary_without_reclassifying_the_operation() {
    use OwnedPreparationClass::*;
    for (class, expected) in [(Declined, 0), (Incomplete, 1), (Failed, 2), (Cancelled, 3)] {
        assert_eq!(tags::local_class(class), expected);
    }
}
#[test]
fn every_unsupported_payload_retains_its_distinct_cause_and_gap_tag() {
    use UnprojectedPreparationPayload::*;
    for (payload, tag) in [
        (Census, 0),
        (ContactTransport, 1),
        (RegionPrism, 2),
        (StageCompletion, 3),
    ] {
        let mut bytes = Vec::new();
        gap(&PrismDiagnosticGap::PreparationCause(payload), &mut bytes);
        cause(
            &OwnedPreparationDiagnosticCause::Unprojected(payload),
            &mut bytes,
        );
        assert_eq!(bytes, [0, tag, 5, tag]);
    }
}
#[test]
fn all_outer_owner_slots_and_missing_callback_are_encoded_without_dropping_gaps() {
    use RetainedPreparationStage::*;
    let owners = vec![
        ConstructedSource,
        BoundaryInputs,
        Census,
        Partition,
        ContactTransport,
        Prism,
    ];
    let mut bytes = Vec::new();
    gap(&PrismDiagnosticGap::OuterOwners(owners), &mut bytes);
    gap(&PrismDiagnosticGap::MissingCallbackEvidence, &mut bytes);
    assert_eq!(bytes, [1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 2]);
}
#[test]
fn unavailable_stage_preserves_the_child_class_and_missing_state_preserves_expected_text() {
    let mut bytes = Vec::new();
    cause(
        &OwnedPreparationDiagnosticCause::StageUnavailable {
            class: OwnedPreparationClass::Failed,
        },
        &mut bytes,
    );
    cause(
        &OwnedPreparationDiagnosticCause::MissingLocalState { expected: "X" },
        &mut bytes,
    );
    assert_eq!(bytes, [3, 2, 4, 1, 0, 0, 0, 0, 0, 0, 0, b'X']);
}
