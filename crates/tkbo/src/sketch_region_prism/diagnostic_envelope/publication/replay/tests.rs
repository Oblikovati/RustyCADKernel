// SPDX-License-Identifier: GPL-2.0-only
//! Primitive format expectations do not construct an invocation or an owned envelope.
use super::*;
#[test]
fn callback_versions_absent_child_and_work_ordinal_have_explicit_literal_bytes() {
    let stop = OwnedPublicationCallback {
        checkpoint: PrismPublicationCheckpoint {
            phase: PrismPublicationPhase::Measures,
            ordinal: 7,
        },
        cause: OwnedDiagnosticCallbackCause::Exhausted,
    };
    let mut old = Vec::new();
    callback_v1(&stop, None, &mut old);
    assert_eq!(old, vec![2, 4, 7, 0, 0, 0, 0, 0, 0, 0, 2, 0]);
    let mut new = Vec::new();
    callback_v2(&stop, None, &mut new);
    assert_eq!(new, vec![3, 4, 7, 0, 0, 0, 0, 0, 0, 0, 2, 0]);
}
#[test]
fn class_origin_and_binding_labels_are_distinct_explicit_operands() {
    let mut bytes = Vec::new();
    origin(PublicationClassOrigin::PublicationOperation, &mut bytes);
    origin(
        PublicationClassOrigin::RetainedStage(PrismPublicationPhase::Commit),
        &mut bytes,
    );
    stage_cause(
        &OwnedPublicationStageCause::Binding { proof: "x" },
        &mut bytes,
    );
    assert_eq!(bytes, vec![0, 1, 5, 1, 1, 0, 0, 0, 0, 0, 0, 0, b'x']);
}
#[test]
fn history_and_all_pending_payloads_use_explicit_tags() {
    let mut bytes = Vec::new();
    cause(
        &OwnedPublicationDiagnosticCause::HistoryCancelled {
            operation: AuthoredPrismId(9),
            phase: PrismHistoryPhase::Commit,
        },
        &mut bytes,
    );
    assert_eq!(bytes, vec![2, 9, 0, 0, 0, 0, 0, 0, 0, 2]);
    for (index, value) in pending_values().into_iter().enumerate() {
        let mut bytes = Vec::new();
        cause(
            &OwnedPublicationDiagnosticCause::Unprojected(value),
            &mut bytes,
        );
        gap(&PublicationDiagnosticGap::Payload(value), &mut bytes);
        assert_eq!(bytes, vec![3, index as u8, 4, index as u8]);
    }
}
fn pending_values() -> [PublicationPendingPayload; 10] {
    use PublicationPendingPayload as P;
    [
        P::JoinClosureQuery,
        P::JoinClosureRegion,
        P::JoinEmbedding,
        P::JoinCapture,
        P::JoinBinding,
        P::JoinLink,
        P::JoinIdentity,
        P::Validation,
        P::History,
        P::Measures,
    ]
}
#[test]
fn scalar_gaps_preserve_actual_expected_and_absent_child_distinctions() {
    let mut bytes = Vec::new();
    gap(&PublicationDiagnosticGap::CallerDictionary, &mut bytes);
    gap(&PublicationDiagnosticGap::RequestedPolicy, &mut bytes);
    gap(
        &PublicationDiagnosticGap::Operation {
            actual: AuthoredPrismId(9),
            expected: AuthoredPrismId(1),
        },
        &mut bytes,
    );
    gap(&PublicationDiagnosticGap::UnretainedChildHeader, &mut bytes);
    assert_eq!(
        bytes,
        vec![0, 1, 2, 9, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3]
    );
}
#[test]
fn unqualified_entity_gap_keeps_full_sweep_role_and_kind() {
    use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenSweepRole};
    let key = FrozenPrismKey {
        operation: AuthoredPrismId(9),
        source: AuthoredPrismId(2),
        role: FrozenSweepRole::Last,
        kind: 4,
    };
    let mut bytes = Vec::new();
    gap(
        &PublicationDiagnosticGap::UnprojectedEntity(key),
        &mut bytes,
    );
    assert_eq!(
        bytes,
        vec![5, 9, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 4]
    );
}
