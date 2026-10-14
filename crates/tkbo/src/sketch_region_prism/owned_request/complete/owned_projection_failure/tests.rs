// SPDX-License-Identifier: GPL-2.0-only
use super::super::operation_tests::{CompleteObserver, accepted};
use super::*;
use crate::sketch_region_prism::owned_request::request_replay::encode_owned_prism_request;
use crate::sketch_region_prism::owned_request::{names::CallerOperationId, test_fixture};
fn stopped(
    request: crate::sketch_region_prism::owned_request::OwnedPolynomialPrismRequest,
    ordinal: usize,
    native: bool,
) -> OwnedProjectionRefusal {
    let mut progress = CompleteObserver {
        stop: (!native).then_some(ordinal),
        fail: native.then_some(ordinal),
        ..Default::default()
    };
    with_owned_prism_operation(request, &mut progress, |result| {
        own_projection_refusal(Box::new(result.err().unwrap()))
            .unwrap_or_else(|_| panic!("actual projection refusal required"))
    })
}
#[test]
fn actual_projection_stops_escape_scope_with_exact_context_and_callback() {
    let (accepted, trace) = accepted();
    for point in &trace.points[trace.points.len() - 6..] {
        for native in [false, true] {
            let value = stopped(accepted.request().clone(), point.ordinal, native);
            assert_stopped(&value, *point, native);
            assert_eq!(
                value.accepted_publication_context().replay_bytes(),
                accepted.publication().replay_bytes()
            );
        }
    }
}
fn assert_stopped(value: &OwnedProjectionRefusal, point: CompletePrismCheckpoint, native: bool) {
    assert_eq!(value.version(), 1);
    assert_eq!(value.phase(), Some(point.phase));
    assert_eq!(value.callback().unwrap().checkpoint, point);
    let class = if native {
        ProjectionRefusalClass::Failed
    } else {
        ProjectionRefusalClass::Cancelled
    };
    assert_eq!(
        value.classification(),
        (class, ProjectionClassOrigin::OuterCallback)
    );
    assert!(
        value
            .replay_bytes()
            .starts_with(b"Martensite.OwnedProjectionRefusal/v1\0")
    );
}
#[test]
fn actual_volume_stop_preserves_child_cancel_beneath_outer_native_failure() {
    let (accepted, trace) = accepted();
    let point = trace
        .points
        .iter()
        .find(|p| p.phase == CompletePrismPhase::Volume)
        .unwrap();
    let value = stopped(accepted.request().clone(), point.ordinal, true);
    assert_eq!(
        value.retained_child_class(),
        RetainedProjectionClass::Volume(
            crate::sketch_region_prism::frozen::joined_volume::JoinedVolumeFailureClass::Cancelled
        )
    );
    assert!(matches!(
        value.callback().unwrap().cause,
        CompletePrismCallbackCause::Failed(_)
    ));
    assert!(matches!(value.cause(), CompleteProjectionCause::Volume(_)));
}
#[test]
fn identical_geometry_with_different_caller_ids_keeps_distinct_actual_context() {
    let (accepted, trace) = accepted();
    let ordinal = trace.points.last().unwrap().ordinal;
    let first = stopped(accepted.request().clone(), ordinal, false);
    let mut request = accepted.request().clone();
    request.operation = CallerOperationId(9001);
    request.sources[0].revision.0 += 1;
    let second = stopped(request, ordinal, false);
    assert_ne!(first.request().operation, second.request().operation);
    assert_eq!(
        second.plan().identities().operation,
        CallerOperationId(9001)
    );
    assert_ne!(first.replay_bytes(), second.replay_bytes());
}
#[test]
fn every_nonprojection_family_returns_the_exact_original_owner() {
    let (_, trace) = accepted();
    let resolution = trace
        .points
        .iter()
        .find(|p| p.phase == CompletePrismPhase::Resolution)
        .unwrap()
        .ordinal;
    for family in 0..3 {
        exercise_nonprojection(family, resolution);
    }
}
fn exercise_nonprojection(family: usize, resolution: usize) {
    let mut request = test_fixture::request();
    if family == 0 {
        request.sources.clear();
    }
    if family == 2 {
        request.policy.publication.version = 17;
    }
    let mut progress = CompleteObserver {
        fail: (family == 1).then_some(resolution),
        ..Default::default()
    };
    with_owned_prism_operation(request, &mut progress, |result| {
        assert_returned(result.err().unwrap())
    });
}
fn owner_address(value: &ScopedPrismFailure<'_>) -> usize {
    match value.cause() {
        ScopedPrismCause::Preparation(value) => (&**value as *const _) as usize,
        ScopedPrismCause::Resolution { cause, .. } => (&**cause as *const _) as usize,
        ScopedPrismCause::Publication(value) => (&**value as *const _) as usize,
        ScopedPrismCause::Projection(_) => panic!("nonprojection fixture required"),
    }
}
fn assert_returned(value: ScopedPrismFailure<'_>) {
    let value = Box::new(value);
    let outer = (&*value as *const ScopedPrismFailure<'_>) as usize;
    let address = owner_address(&value); // Test-only move identity, never persistent naming.
    let request = encode_owned_prism_request(value.request());
    let callback = value.callback().map(|c| c.checkpoint);
    let returned = own_projection_refusal(value).err().unwrap();
    assert_eq!(
        outer,
        (&*returned as *const ScopedPrismFailure<'_>) as usize
    );
    assert_eq!(address, owner_address(&returned));
    assert_eq!(callback, returned.callback().map(|c| c.checkpoint));
    assert_eq!(request, encode_owned_prism_request(returned.request()));
}
#[test]
fn owned_projection_types_are_send_sync_without_mutable_owners() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<OwnedProjectionRefusal>();
    send_sync::<CompleteProjectionCause>();
    send_sync::<ProjectionRefusalClass>();
    send_sync::<ProjectionClassOrigin>();
    send_sync::<RetainedProjectionClass>();
}
#[test]
fn projection_bytes_are_independent_across_threads_and_lock_free_readers() {
    let (accepted, trace) = accepted();
    let ordinal = trace.points.last().unwrap().ordinal;
    let request = accepted.request().clone();
    let value = stopped(request.clone(), ordinal, true);
    let expected = value.replay_bytes();
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| value.replay_bytes()))
            .collect();
        let workers: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| stopped(request.clone(), ordinal, true).replay_bytes()))
            .collect();
        for task in readers.into_iter().chain(workers) {
            assert_eq!(task.join().unwrap(), expected);
        }
    });
}
#[test]
fn owned_projection_refusal_process_receipt() {
    let (accepted, trace) = accepted();
    let mut bytes = Vec::new();
    for point in &trace.points[trace.points.len() - 6..] {
        for native in [false, true] {
            let value = stopped(accepted.request().clone(), point.ordinal, native);
            crate::sketch_region_prism::frozen::proof_join::encode_joined_block(
                &value.replay_bytes(),
                &mut bytes,
            );
        }
    }
    if let Some(path) = std::env::var_os("MARTENSITE_PROJECTION_REFUSAL_RECEIPT") {
        std::fs::write(path, bytes).unwrap();
    }
}

#[test]
fn retained_request_and_publication_survive_source_mutation_and_drop() {
    let (accepted, trace) = accepted();
    let mut source = accepted.request().clone();
    let owned = stopped(source.clone(), trace.points.last().unwrap().ordinal, true);
    let expected = owned.replay_bytes();
    source.sources.clear();
    source.operation = CallerOperationId(u64::MAX);
    drop(source);
    drop(accepted);
    assert_eq!(owned.request().sources.len(), 1);
    assert_eq!(owned.replay_bytes(), expected);
}
