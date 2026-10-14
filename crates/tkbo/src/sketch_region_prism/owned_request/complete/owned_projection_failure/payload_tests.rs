// SPDX-License-Identifier: GPL-2.0-only
//! Named fixture injects owned typed causes only; actual invocation tests cover producing callbacks.
use super::*;
use crate::sketch_region_prism::{
    frozen::{AuthoredPrismId, FrozenOriginalKey, joined_volume::*},
    owned_request::{
        names::{body_map::*, reservations::*, *},
        resolver::fixture::invoked_prism_fixture,
        test_fixture,
    },
    publication::body_view::*,
};
struct ProjectionPayloadFixture;
impl ProjectionPayloadFixture {
    fn project(cause: CompleteProjectionCause) -> OwnedProjectionRefusal {
        let original = Box::new(ProjectionRefusal {
            invoked: invoked_prism_fixture(test_fixture::request()),
            cause,
        });
        let address = (&*original as *const ProjectionRefusal) as usize;
        let failure = ScopedPrismFailure {
            callback: None,
            cause: ScopedPrismCause::Projection(original),
        };
        let owned = own_projection_refusal(Box::new(failure))
            .unwrap_or_else(|_| panic!("projection branch"));
        assert_eq!(
            address,
            (&*owned.original as *const ProjectionRefusal) as usize
        );
        owned
    }
}
fn causes() -> Vec<CompleteProjectionCause> {
    vec![
        CompleteProjectionCause::Callback,
        CompleteProjectionCause::Names(Box::new(PrismSemanticMapFailure {
            operation: CallerOperationId(257),
            cause: PrismSemanticMapCause::UnknownToken { actual: 999 },
        })),
        CompleteProjectionCause::Reservations(PrismReservationFailure {
            operation: CallerOperationId(257),
            cause: PrismReservationCause::ForeignMap,
        }),
        CompleteProjectionCause::Body(Box::new(body_failure())),
        CompleteProjectionCause::Volume(Box::new(volume_failure())),
    ]
}
fn body_failure() -> PrismBodyProjectionFailure {
    PrismBodyProjectionFailure {
        operation: CallerOperationId(257),
        source: CallerBoundaryId(9),
        version: 1,
        entity: None,
        class: PrismBodyProjectionClass::Incomplete,
        cause: PrismBodyProjectionCause::PublicationBinding,
    }
}
fn volume_failure() -> JoinedPrismVolumeFailure {
    JoinedPrismVolumeFailure {
        operation: AuthoredPrismId(257),
        requested_version: 1,
        entity: FrozenOriginalKey {
            source: 1,
            member: AuthoredPrismId(513),
            directed_use: AuthoredPrismId(519),
        },
        class: JoinedVolumeFailureClass::Incomplete,
        cause: JoinedPrismVolumeCause::Version { expected: 1 },
    }
}
#[test]
fn every_owned_cause_moves_without_reconstruction_and_keeps_class_provenance() {
    for (index, cause) in causes().into_iter().enumerate() {
        let owned = ProjectionPayloadFixture::project(cause);
        let origin = if index < 3 {
            ProjectionClassOrigin::BridgePolicyV1
        } else {
            ProjectionClassOrigin::RetainedChild
        };
        assert_eq!(
            owned.classification(),
            (ProjectionRefusalClass::Incomplete, origin)
        );
        assert!(owned.callback().is_none());
        assert_eq!(owned.phase(), expected_phase(index));
        assert_eq!(owned.retained_child_class(), expected_class(index));
        assert!(!owned.replay_bytes().is_empty());
    }
}
fn expected_phase(index: usize) -> Option<CompletePrismPhase> {
    [
        None,
        Some(CompletePrismPhase::Names),
        Some(CompletePrismPhase::Reservations),
        Some(CompletePrismPhase::Body),
        Some(CompletePrismPhase::Volume),
    ][index]
}
fn expected_class(index: usize) -> RetainedProjectionClass {
    match index {
        0..=2 => RetainedProjectionClass::Unavailable,
        3 => RetainedProjectionClass::Body(PrismBodyProjectionClass::Incomplete),
        _ => RetainedProjectionClass::Volume(JoinedVolumeFailureClass::Incomplete),
    }
}
#[test]
fn full_replay_contains_only_exact_request_plan_publication_and_failure() {
    let owned = ProjectionPayloadFixture::project(CompleteProjectionCause::Callback);
    let bytes = owned.replay_bytes();
    let mut rest = &bytes[b"Martensite.OwnedProjectionRefusal/v1\0".len()..];
    let request =
        crate::sketch_region_prism::owned_request::request_replay::encode_owned_prism_request(
            owned.request(),
        );
    assert_eq!(take_block(&mut rest), request);
    let context = SemanticReplayEnvelope::new(
        owned.plan().clone(),
        owned.accepted_publication_context().replay_bytes(),
    );
    assert_eq!(take_block(&mut rest), context.replay_bytes());
    assert_eq!(rest, [0, 2, 2, 0, 0, 0]); // coverage, class, origin, missing child class, absent callback, cause.
}
fn take_block<'a>(bytes: &mut &'a [u8]) -> &'a [u8] {
    let length = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
    let result = &bytes[8..8 + length];
    *bytes = &bytes[8 + length..];
    result
}

#[test]
fn retained_child_cancellation_and_outer_exhaustion_have_distinct_origins() {
    let mut cause = volume_failure();
    cause.class = JoinedVolumeFailureClass::Cancelled;
    cause.cause = JoinedPrismVolumeCause::Cancelled(VolumeCheckpoint::Begin);
    let mut owned =
        ProjectionPayloadFixture::project(CompleteProjectionCause::Volume(Box::new(cause)));
    assert_child_cancellation(&owned);
    owned.callback = Some(exhausted_stop());
    assert_eq!(
        owned.classification(),
        (
            ProjectionRefusalClass::Incomplete,
            ProjectionClassOrigin::OuterCallback
        )
    );
    assert_eq!(
        owned.retained_child_class(),
        RetainedProjectionClass::Volume(JoinedVolumeFailureClass::Cancelled)
    );
}
fn assert_child_cancellation(owned: &OwnedProjectionRefusal) {
    let expected = (
        ProjectionRefusalClass::Cancelled,
        ProjectionClassOrigin::RetainedChild,
    );
    assert_eq!(owned.classification(), expected);
}
fn exhausted_stop() -> CompletePrismCallbackStop {
    CompletePrismCallbackStop {
        checkpoint: CompletePrismCheckpoint {
            phase: CompletePrismPhase::Volume,
            ordinal: usize::MAX,
            child_ordinal: None,
        },
        cause: CompletePrismCallbackCause::Exhausted,
    }
}
