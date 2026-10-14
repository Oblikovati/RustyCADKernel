// SPDX-License-Identifier: GPL-2.0-only
//! Exhaustive adapters preserve the old live causes; pure types admit no mutable diagnostic variants.
use super::*;
fn proof() -> PrismOrientationEvidence {
    let (bridge, shapes) = fixture();
    oriented(&bridge, &shapes, &run(&bridge, &shapes))
}
#[test]
fn pure_unsupported_cause_preserves_exact_occurrence_and_orientation() {
    let entity = Box::new(proof().faces()[0].domain().face.clone());
    let legacy = PrismOrientationCause::from(PrismOrientationEvaluationCause::Unsupported {
        entity: entity.clone(),
        actual: Orientation::External,
    });
    assert!(
        matches!(legacy, PrismOrientationCause::Unsupported { entity: actual, actual: direction }
        if actual == entity && direction == Orientation::External)
    );
}
#[test]
fn pure_winding_and_zero_causes_preserve_exact_legacy_variants() {
    let legacy = PrismOrientationCause::from(PrismOrientationEvaluationCause::Winding {
        actual: -1,
        expected: 1,
    });
    assert!(matches!(
        legacy,
        PrismOrientationCause::Winding {
            actual: -1,
            expected: 1
        }
    ));
    let legacy = PrismOrientationCause::from(PrismOrientationEvaluationCause::ZeroJacobian);
    assert!(matches!(legacy, PrismOrientationCause::ZeroJacobian));
}
#[test]
fn pure_face_cause_preserves_complete_domain_and_expected_direction() {
    let domain = Box::new(proof().faces()[0].domain().clone());
    let legacy = PrismOrientationCause::from(PrismOrientationEvaluationCause::Face {
        domain: domain.clone(),
        actual: Orientation::Forward,
        expected: Orientation::Reversed,
    });
    assert!(
        matches!(legacy, PrismOrientationCause::Face { domain: actual, actual: direction, expected }
        if actual == domain && direction == Orientation::Forward && expected == Orientation::Reversed)
    );
}
#[test]
fn pure_trim_cause_preserves_complete_authored_use_and_expected_direction() {
    let authored = Box::new(proof().faces()[0].trims()[0].authored().clone());
    let legacy = PrismOrientationCause::from(PrismOrientationEvaluationCause::Trim {
        authored: authored.clone(),
        actual: Orientation::Forward,
        expected: Orientation::Reversed,
    });
    assert!(
        matches!(legacy, PrismOrientationCause::Trim { authored: actual, actual: direction, expected }
        if actual == authored && direction == Orientation::Forward && expected == Orientation::Reversed)
    );
}
#[test]
fn pure_failure_adapter_keeps_legacy_policy_phase_and_original() {
    let evidence = proof();
    for (phase, expected) in [
        (
            PrismOrientationEvaluationPhase::Winding,
            PrismOrientationPhase::Winding,
        ),
        (
            PrismOrientationEvaluationPhase::Faces,
            PrismOrientationPhase::Faces,
        ),
    ] {
        check_phase(&evidence, phase, expected);
    }
}
#[test]
fn pure_orientation_errors_are_owned_send_sync_values() {
    static_assertions::assert_impl_all!(PrismOrientationEvaluationPhase: Send, Sync);
    static_assertions::assert_impl_all!(PrismOrientationEvaluationCause: Send, Sync);
    static_assertions::assert_impl_all!(PrismOrientationEvaluationFailure: Send, Sync);
    static_assertions::assert_impl_all!(super::super::super::super::frozen::joined_orientation::JoinedPrismOrientationFailure: Send, Sync);
    static_assertions::assert_impl_all!(super::super::super::super::frozen::joined_orientation::JoinedPrismOrientationCause: Send, Sync);
}

fn check_phase(
    evidence: &PrismOrientationEvidence,
    phase: PrismOrientationEvaluationPhase,
    expected: PrismOrientationPhase,
) {
    let pure = PrismOrientationEvaluationFailure {
        phase,
        cause: PrismOrientationEvaluationCause::ZeroJacobian,
    };
    let legacy = evaluation_error::legacy_failure(options(), evidence.embedding(), pure);
    assert_eq!(legacy.requested, options());
    assert_eq!(legacy.phase, expected);
    assert_eq!(
        legacy.original.as_deref(),
        Some(&evidence.embedding().binding().region.original)
    );
    assert!(matches!(legacy.cause, PrismOrientationCause::ZeroJacobian));
}
