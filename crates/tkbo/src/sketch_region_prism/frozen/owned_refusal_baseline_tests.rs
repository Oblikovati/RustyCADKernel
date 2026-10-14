// SPDX-License-Identifier: GPL-2.0-only
//! Pin the local refusal before adding an optional immutable diagnostic projection.
use super::{proof_join::*, *};
use crate::sketch_region_prism::region_certificate::current_binding::PrismBindingFailure;
use crate::sketch_region_prism::{PrismCause, PrismCompletion, PrismPhase};
use rustycad_tkernel::standard::Failure;

fn native_refusal() -> Failure {
    let cause = Failure::with_type_bytes("Standard_DomainError", &[255, 65], &[254, 66]);
    cause.with_exception_parent("Standard_Failure")
}

fn nested_refusal() -> PrismProofJoinFailure {
    let cause = PrismCause::Mapped {
        phase: PrismPhase::Sweep,
        failure: native_refusal(),
    };
    PrismProofJoinFailure {
        operation: AuthoredPrismId(401),
        phase: PrismProofJoinPhase::Embedding,
        entity: None,
        cause: PrismProofJoinCause::Binding(Box::new(PrismBindingFailure::Body(
            PrismCompletion::Failed(cause),
        ))),
    }
}

#[test]
fn owned_refusal_baseline_retains_nested_native_bytes_and_phase() {
    let failure = nested_refusal();
    let PrismProofJoinCause::Binding(binding) = failure.cause else {
        panic!("expected binding")
    };
    let PrismBindingFailure::Body(PrismCompletion::Failed(PrismCause::Mapped { phase, failure })) =
        *binding
    else {
        panic!("expected retained native body failure")
    };
    assert_eq!(phase, PrismPhase::Sweep);
    assert_eq!(failure, native_refusal());
    assert_eq!(failure.message_bytes(), &[255, 65]);
    assert_eq!(failure.stack_bytes(), &[254, 66]);
}

fn capture_refusal() -> PrismFreezeFailure {
    PrismFreezeFailure {
        operation: AuthoredPrismId(401),
        phase: FreezePhase::Geometry,
        entity: None,
        cause: PrismFreezeCause::Native(native_refusal()),
    }
}

#[test]
fn owned_refusal_baseline_distinguishes_failed_and_incomplete_capture() {
    let failed = PrismProofJoinCause::FailedCapture(Box::new(capture_refusal()));
    let incomplete = PrismProofJoinCause::IncompleteCapture(Box::new(capture_refusal()));
    assert!(matches!(failed, PrismProofJoinCause::FailedCapture(_)));
    assert!(matches!(
        incomplete,
        PrismProofJoinCause::IncompleteCapture(_)
    ));
}

#[test]
fn owned_refusal_baseline_retains_nested_cancel_phase() {
    let cancelled = PrismProofJoinCause::Cancelled {
        freeze_phase: Some(FreezePhase::Commit),
    };
    assert!(matches!(
        cancelled,
        PrismProofJoinCause::Cancelled {
            freeze_phase: Some(FreezePhase::Commit)
        }
    ));
}
