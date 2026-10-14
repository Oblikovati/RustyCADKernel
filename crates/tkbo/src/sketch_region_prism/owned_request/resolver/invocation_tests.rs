// SPDX-License-Identifier: GPL-2.0-only
use super::{source_tests::*, *};
use crate::sketch_region_prism::publication::*;
use rustycad_tkernel::standard::Failure;
#[derive(Default)]
pub(super) struct PublicationObserver {
    pub seen: Vec<PrismPublicationCheckpoint>,
    pub stop: Option<usize>,
    pub fail: Option<usize>,
}
impl PrismPublicationProgress for PublicationObserver {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point);
        if self.fail == Some(point.ordinal) {
            return Err(Failure::new(
                "resolver publication callback failure; expected continuation",
            ));
        }
        Ok(self.stop == Some(point.ordinal))
    }
}
#[test]
fn actual_owned_request_resolver_reaches_complete_private_publication() {
    let prepared = operation(false);
    let invocation = checked(invocation::resolve_invocation(
        &prepared,
        plan_for(&prepared),
        &mut || false,
    ));
    let mut progress = PublicationObserver::default();
    let result = invocation
        .publish(&mut progress)
        .unwrap_or_else(|failure| publication_refusal(&failure));
    assert_accepted_result(&result);
    assert!(progress.seen.len() > 1);
}
fn assert_accepted_result(result: &invocation::ResolvedPrismPublication) {
    assert_eq!(
        result.result().completion(),
        PrismPublicationCompletion::CompleteNamedPrismV1
    );
    assert_eq!(result.result().join().projection().nodes().len(), 13);
    assert_eq!(result.result().join().projection().uses().len(), 26);
    assert_eq!(result.plan().version(), 2);
    assert_eq!(result.request().operation.0, 257);
    let names = body_map::issue_resolved_prism_names(result).unwrap();
    assert!(names.binds(result.result()));
    assert_eq!(
        (names.member_bindings().len(), names.use_bindings().len()),
        (13, 26)
    );
}

fn publication_refusal(failure: &invocation::InvokedPrismPublicationFailure<'_>) -> ! {
    let actual = failure.failure();
    if let PrismPublicationCause::Stage(stage) = &actual.cause
        && let PublicationStageCause::Join(join) = &stage.cause
    {
        join_refusal(join);
    }
    panic!("publication {:?} {:?}", actual.phase, actual.class)
}
fn join_refusal(join: &crate::sketch_region_prism::frozen::proof_join::PrismProofJoinFailure) -> ! {
    use crate::sketch_region_prism::frozen::proof_join::PrismProofJoinCause;
    match &join.cause {
        PrismProofJoinCause::IncompleteCapture(cause)
        | PrismProofJoinCause::FailedCapture(cause) => panic!("capture {:?}", cause),
        PrismProofJoinCause::Link(cause) => panic!("link {:?}", cause),
        _ => panic!("join phase {:?}", join.phase),
    }
}

#[test]
fn actual_issued_map_preserves_cross_caller_namespace_without_internal_token_reservation() {
    let first = fixture::default_resolved_prism_fixture();
    let mut request = first.request.clone();
    request.operation = CallerOperationId(2056);
    request.sources[0].source = CallerBoundaryId(2336);
    request.sources[0].native_start = CallerDirectedUseId(19);
    request.sources[0].native_end = CallerDirectedUseId(901);
    let second = fixture::resolved_prism_fixture(request);
    assert_eq!(
        first.publication.replay_bytes(),
        second.publication.replay_bytes()
    );
    assert_ne!(first.names.replay_bytes(), second.names.replay_bytes());
    assert_second_caller_names(&second.names);
}
#[test]
fn sealed_failure_packets_keep_distinct_callers_when_internal_tokens_are_identical() {
    let first = cancelled_packet_request(700);
    let second = cancelled_packet_request(900);
    assert_eq!(first.operation.0, 700);
    assert_eq!(second.operation.0, 900);
}
fn cancelled_packet_request(operation: u64) -> super::super::request::OwnedPolynomialPrismRequest {
    let prepared = prepared_caller(operation);
    let invocation = checked(invocation::resolve_invocation(
        &prepared,
        plan_for(&prepared),
        &mut || false,
    ));
    let mut progress = stopping_observer();
    let packet = invocation.publish(&mut progress).err().unwrap();
    let owned = packet.request().clone();
    assert_eq!(packet.plan().identities().operation, owned.operation);
    let bindings = packet.bindings();
    assert_eq!(bindings.operation.0, 257);
    let failure = packet.into_failure();
    assert_eq!(failure.operation, bindings.operation);
    assert_eq!(failure.class, PrismPublicationClass::Cancelled);
    owned
}
#[test]
fn accepted_scalar_request_and_map_owners_are_send_sync_and_isolated() {
    fn check<T: Send + Sync>() {}
    check::<invocation::ResolvedPrismPublication>();
    check::<fixture::ResolvedPrismFixture>();
    let result = fixture::default_resolved_prism_fixture();
    let before = result.names.replay_bytes().to_vec();
    let mut caller = result.request.clone();
    caller.sources[0].world_poles[1][0] = 8.;
    caller.sources[0].revision.0 += 1;
    assert_eq!(result.request.sources[0].world_poles[1][0], 4.);
    assert_eq!(result.names.replay_bytes(), before);
    assert!(result.names.binds(&result.publication));
}

fn assert_second_caller_names(names: &body_map::ResolvedPrismNames) {
    assert!(
        names
            .member_bindings()
            .iter()
            .all(|row| row.semantic.operation.0 == 2056 && row.semantic.source.0 == 2336)
    );
}

fn prepared_caller(operation: u64) -> PreparedPolynomialPrism {
    let mut request = super::super::test_fixture::request();
    request.operation = CallerOperationId(operation);
    super::super::test_fixture::prepared(request)
}

fn stopping_observer() -> PublicationObserver {
    PublicationObserver {
        stop: Some(0),
        ..PublicationObserver::default()
    }
}
