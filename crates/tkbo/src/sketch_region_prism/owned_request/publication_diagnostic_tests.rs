// SPDX-License-Identifier: GPL-2.0-only
//! Every envelope in this corpus comes from actual preparation/resolution/publication.
use super::{names::*, resolver::resolve_invocation, *};
use crate::sketch_region_prism::{diagnostic_envelope::*, publication::*};
use rustycad_tkernel::standard::Failure;
#[derive(Default)]
struct NamedPublicationObserver {
    failure: Option<Failure>,
    cancelled: bool,
}
impl PrismPublicationProgress for NamedPublicationObserver {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        Ok(self.cancelled)
    }
}
fn request(version: u32) -> OwnedPolynomialPrismRequest {
    let mut request = test_fixture::request();
    request.policy.publication.version = version;
    request
}
fn with_refusal<T>(
    request: OwnedPolynomialPrismRequest,
    observer: &mut NamedPublicationObserver,
    inspect: impl for<'a> FnOnce(Box<InvokedPrismPublicationFailure<'a>>) -> T,
) -> T {
    let plan = SingleBoundaryNamePlan::v2(
        request.sources[0].names(request.operation),
        request.selection,
    )
    .unwrap();
    let prepared = test_fixture::prepared(request);
    let invocation = resolve_invocation(&prepared, plan, &mut || false)
        .unwrap_or_else(|_| panic!("expected resolved invocation"));
    let packet = invocation
        .publish(observer)
        .err()
        .expect("expected actual refusal");
    inspect(packet)
}
fn native_observer() -> NamedPublicationObserver {
    NamedPublicationObserver {
        failure: Some(Failure::out_of_range("owned actual publication observer")),
        cancelled: false,
    }
}
fn project_request(request: OwnedPolynomialPrismRequest) -> Box<OwnedPrismPublicationDiagnostic> {
    with_refusal(request, &mut native_observer(), |packet| {
        project_invoked_publication_refusal(packet).into_parts().0
    })
}
#[test]
fn actual_v2_native_refusal_is_owned_and_keeps_original_and_child_classes() {
    let diagnostic = project_request(request(2));
    assert_eq!(
        diagnostic.coverage(),
        DiagnosticProjectionCoverage::Complete
    );
    assert_eq!(diagnostic.class(), PrismPublicationClass::Failed);
    let OwnedPublicationDiagnosticCause::Stage(stage) = diagnostic.cause() else {
        panic!("expected stage")
    };
    let OwnedPublicationStageCause::CallbackV2 { stop, downstream } = &stage.cause else {
        panic!("expected V2 callback")
    };
    assert_native_stop(stop);
    let child = downstream.as_ref().unwrap();
    assert_eq!(child.class, PrismPublicationClass::Incomplete);
    let origin = PublicationClassOrigin::RetainedStage(PrismPublicationPhase::Join);
    assert_join_origin(&child.cause, origin);
}
fn assert_join_origin(cause: &OwnedPublicationStageCause, expected: PublicationClassOrigin) {
    let OwnedPublicationStageCause::Join {
        evidence,
        class_origin,
        encoded,
    } = cause
    else {
        panic!("expected projected join")
    };
    assert_eq!(*class_origin, expected);
    assert!(!encoded.is_empty());
    assert_eq!(evidence.operation().0, 257);
}
#[test]
fn actual_v1_gap_preserves_same_original_box_and_never_infers_child_class() {
    with_refusal(request(1), &mut native_observer(), |packet| {
        let original = packet.failure() as *const _;
        let (diagnostic, retained) = project_invoked_publication_refusal(packet).into_parts();
        assert_eq!(retained.as_deref().unwrap() as *const _, original);
        assert_eq!(diagnostic.class(), PrismPublicationClass::Failed);
        assert_eq!(
            diagnostic.gaps(),
            &[PublicationDiagnosticGap::UnretainedChildHeader]
        );
        assert_legacy_origin(diagnostic.cause());
    });
}
fn assert_legacy_origin(cause: &OwnedPublicationDiagnosticCause) {
    let OwnedPublicationDiagnosticCause::Stage(stage) = cause else {
        panic!("expected stage")
    };
    let OwnedPublicationStageCause::CallbackV1 { downstream, .. } = &stage.cause else {
        panic!("expected V1 callback")
    };
    assert_join_origin(
        downstream.as_ref().unwrap(),
        PublicationClassOrigin::PublicationOperation,
    );
}
#[test]
fn unsupported_publication_version_is_exact_complete_diagnostic_without_body() {
    let diagnostic = project_request(request(17));
    assert_eq!(
        diagnostic.coverage(),
        DiagnosticProjectionCoverage::Complete
    );
    assert_eq!(diagnostic.phase(), PrismPublicationPhase::Admission);
    assert_eq!(diagnostic.class(), PrismPublicationClass::Incomplete);
    assert!(matches!(
        diagnostic.cause(),
        OwnedPublicationDiagnosticCause::UnsupportedVersion {
            actual: 17,
            expected: 1
        }
    ));
}
#[test]
fn unsupported_validation_payload_keeps_same_original_with_honest_gap() {
    let mut requested = request(2);
    requested.policy.publication.validation.version = 17;
    with_refusal(
        requested,
        &mut NamedPublicationObserver::default(),
        |packet| {
            let original = packet.failure() as *const _;
            let (diagnostic, retained) = project_invoked_publication_refusal(packet).into_parts();
            assert_eq!(retained.as_deref().unwrap() as *const _, original);
            assert_eq!(diagnostic.class(), PrismPublicationClass::Incomplete);
            assert_eq!(
                diagnostic.gaps(),
                &[PublicationDiagnosticGap::Payload(
                    PublicationPendingPayload::Validation
                )]
            );
        },
    );
}
#[test]
fn identical_internal_tokens_do_not_alias_distinct_caller_contexts() {
    let first = project_request(request(2));
    let mut other = request(2);
    other.operation = CallerOperationId(900);
    other.sources[0].source = CallerBoundaryId(700);
    let second = project_request(other);
    assert_eq!(first.operation(), second.operation());
    assert_ne!(
        first.context().request().operation,
        second.context().request().operation
    );
    assert_ne!(first.replay_bytes(), second.replay_bytes());
}
#[test]
fn owned_refusal_remains_unchanged_after_caller_mutation_and_owner_drop() {
    static_assertions::assert_impl_all!(OwnedPrismPublicationDiagnostic: Send, Sync);
    let mut caller = request(2);
    let diagnostic = project_request(caller.clone());
    let before = diagnostic.replay_bytes();
    caller.sources[0].world_poles[1][0] = 9.;
    caller.sources[0].revision.0 += 1;
    assert_eq!(diagnostic.replay_bytes(), before);
    assert_eq!(
        diagnostic.context().request().sources[0].world_poles[1][0],
        4.
    );
}

pub(in crate::sketch_region_prism) fn with_diagnostic_context<T>(
    inspect: impl FnOnce(
        &crate::sketch_region_prism::frozen::PrismPublicationBindings,
        PrismPublicationOptions,
    ) -> T,
) -> T {
    with_refusal(
        request(17),
        &mut NamedPublicationObserver::default(),
        |packet| inspect(packet.bindings(), packet.failure().requested),
    )
}
#[path = "publication_diagnostic_replay_tests.rs"]
mod replay_tests;
fn assert_native_stop(stop: &OwnedPublicationCallback) {
    assert_eq!(stop.checkpoint.ordinal, 0);
    assert_eq!(
        stop.cause,
        OwnedDiagnosticCallbackCause::Failed(Failure::out_of_range(
            "owned actual publication observer"
        ))
    );
}

/// ADR0056 follow-up: an attributed history finding without a local owner is now moved whole
/// into the owned diagnostic; its exact typed operands and codec bytes replace the former gap.
#[test]
fn actual_history_refusal_moves_its_complete_finding_into_owned_evidence() {
    let mut history = request(2);
    history.policy.publication.history.version = 17;
    let mut observer = NamedPublicationObserver::default();
    with_refusal(history, &mut observer, |packet| {
        let (diagnostic, retained) = project_invoked_publication_refusal(packet).into_parts();
        assert!(retained.is_none() && diagnostic.gaps().is_empty());
        assert_eq!(diagnostic.phase(), PrismPublicationPhase::History);
        assert_owned_history(&diagnostic);
    });
}
fn assert_owned_history(diagnostic: &OwnedPrismPublicationDiagnostic) {
    let OwnedPublicationDiagnosticCause::History(owned) = diagnostic.cause() else {
        panic!("expected owned history finding");
    };
    let cause = format!("{:?}", owned.finding().cause);
    assert!(
        cause.contains("complete history behavior version"),
        "{cause}"
    );
    let tag = b"Martensite.PublicationHistoryDiagnostic/v1\0";
    assert!(owned.encoded().starts_with(tag));
    let (replay, encoded) = (diagnostic.replay_bytes(), owned.encoded());
    assert!(replay.windows(encoded.len()).any(|w| w == encoded));
}
#[test]
fn actual_measure_refusals_keep_exact_originals_and_typed_family_gaps() {
    let mut measures = request(2);
    measures.policy.publication.measures.version = 17;
    assert_actual_gap(
        measures,
        PublicationPendingPayload::Measures,
        PrismPublicationPhase::Measures,
    );
}
fn assert_actual_gap(
    request: OwnedPolynomialPrismRequest,
    gap: PublicationPendingPayload,
    phase: PrismPublicationPhase,
) {
    with_refusal(
        request,
        &mut NamedPublicationObserver::default(),
        |packet| {
            let original = packet.failure() as *const _;
            let (diagnostic, retained) = project_invoked_publication_refusal(packet).into_parts();
            assert_eq!(retained.as_deref().unwrap() as *const _, original);
            assert_gap_header(&diagnostic, gap, phase);
        },
    );
}
#[test]
fn actual_join_and_embedding_version_refusals_project_without_native_owner() {
    let mut join = request(2);
    join.policy.publication.join.version = 17;
    assert_complete_version(join);
    let mut embedding = request(2);
    embedding.policy.publication.join.embedding.version = 17;
    assert_complete_version(embedding);
}
fn assert_complete_version(request: OwnedPolynomialPrismRequest) {
    with_refusal(
        request,
        &mut NamedPublicationObserver::default(),
        |packet| {
            let (diagnostic, retained) = project_invoked_publication_refusal(packet).into_parts();
            assert!(retained.is_none());
            assert_eq!(
                diagnostic.coverage(),
                DiagnosticProjectionCoverage::Complete
            );
            assert_eq!(diagnostic.class(), PrismPublicationClass::Incomplete);
        },
    );
}
fn assert_gap_header(
    value: &OwnedPrismPublicationDiagnostic,
    gap: PublicationPendingPayload,
    phase: PrismPublicationPhase,
) {
    assert_eq!(value.coverage(), DiagnosticProjectionCoverage::Incomplete);
    assert_eq!(value.class(), PrismPublicationClass::Incomplete);
    assert_eq!(value.phase(), phase);
    assert_eq!(value.gaps(), &[PublicationDiagnosticGap::Payload(gap)]);
}
