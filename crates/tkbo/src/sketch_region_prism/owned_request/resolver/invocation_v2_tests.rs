// SPDX-License-Identifier: GPL-2.0-only
//! Actual invocation dispatch preserves the original request and distinct child/operation classes.
use super::*;
use crate::sketch_region_prism::owned_request::{resolver::source_tests::*, test_fixture};
use crate::sketch_region_prism::{
    frozen::proof_join::{PrismProofJoinCause, PrismProofJoinPhase},
    region_certificate::{current_binding::PrismBindingFailure, embedding::*},
};
use rustycad_tkernel::standard::Failure;

#[derive(Default)]
struct FailingInvocationObserver {
    seen: usize,
}
impl PrismPublicationProgress for FailingInvocationObserver {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.seen += 1;
        Err(Failure::out_of_range("actual invocation callback failure"))
    }
}
fn prepared_version(version: u32) -> PreparedPolynomialPrism {
    let mut request = test_fixture::request();
    request.policy.publication.version = version;
    test_fixture::prepared(request)
}
#[test]
fn actual_v2_packet_preserves_request_and_distinct_child_class() {
    let prepared = prepared_version(2);
    let invocation = checked(resolve_invocation(
        &prepared,
        plan_for(&prepared),
        &mut || false,
    ));
    let mut observer = FailingInvocationObserver::default();
    let packet = invocation.publish(&mut observer).err().unwrap();
    assert_eq!(packet.request().policy.publication.version, 2);
    assert_eq!(packet.failure().requested.version, 2);
    assert_eq!(packet.failure().class, PrismPublicationClass::Failed);
    assert_eq!(observer.seen, 1);
    assert_actual_child(packet.into_failure().cause);
}
fn assert_actual_child(cause: PrismPublicationCause<'_>) {
    let PrismPublicationCause::Stage(stage) = cause else {
        panic!("expected actual failed stage")
    };
    let PublicationStageCause::CallbackWithChild { stop, downstream } = stage.cause else {
        panic!("expected V2 child retention")
    };
    let PublicationCallbackCause::Failed(cause) = stop.cause else {
        panic!("expected native callback failure")
    };
    assert_eq!(
        cause,
        Failure::out_of_range("actual invocation callback failure")
    );
    let child = downstream.unwrap();
    // Existing source::join_failure classifies nested embedding causes as Incomplete.
    assert_eq!(child.class, PrismPublicationClass::Incomplete);
    assert_eq!(child.phase, PrismPublicationPhase::Join);
    assert_nested_binding_cancellation(child.cause);
}
fn assert_nested_binding_cancellation(cause: PublicationStageCause) {
    let PublicationStageCause::Join(join) = cause else {
        panic!("expected join child")
    };
    assert_eq!(join.phase, PrismProofJoinPhase::Embedding);
    let PrismProofJoinCause::Embedding(embedding) = join.cause else {
        panic!("expected embedding cause")
    };
    assert_eq!(embedding.phase, PrismEmbeddingPhase::CurrentBinding);
    let PrismEmbeddingCause::Binding(binding) = embedding.cause else {
        panic!("expected binding cause")
    };
    assert!(matches!(*binding, PrismBindingFailure::Cancelled));
}
#[test]
fn unsupported_invocation_version_keeps_legacy_admission_and_skips_callback() {
    let prepared = prepared_version(17);
    let invocation = checked(resolve_invocation(
        &prepared,
        plan_for(&prepared),
        &mut || false,
    ));
    let mut observer = FailingInvocationObserver::default();
    let packet = invocation.publish(&mut observer).err().unwrap();
    assert_eq!(packet.request().policy.publication.version, 17);
    assert_eq!(packet.failure().class, PrismPublicationClass::Incomplete);
    assert!(matches!(
        packet.failure().cause,
        PrismPublicationCause::UnsupportedVersion {
            actual: 17,
            expected: 1
        }
    ));
    assert_eq!(observer.seen, 0);
}
#[test]
fn complete_actual_v2_invocation_still_issues_the_same_semantic_domain() {
    let mut request = test_fixture::request();
    request.policy.publication.version = 2;
    let result = super::super::fixture::resolved_prism_fixture(request);
    assert_eq!(
        result.publication.completion(),
        PrismPublicationCompletion::CompleteNamedPrismV2
    );
    assert!(result.names.binds(&result.publication));
    assert_eq!(result.names.member_bindings().len(), 13);
    assert_eq!(result.names.use_bindings().len(), 26);
}
