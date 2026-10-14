// SPDX-License-Identifier: GPL-2.0-only
//! Actual accepted operation replay; these receipts supplement the unchanged symbolic baseline.
use super::*;
use crate::sketch_region_prism::owned_request::*;

fn actual_receipt() -> Vec<u8> {
    actual_tests::actual_body().replay_bytes()
}
#[test]
fn actual_body_process_receipt() {
    let bytes = actual_receipt();
    if let Some(path) = std::env::var_os("MARTENSITE_ACTUAL_BODY_RECEIPT") {
        std::fs::write(path, bytes).unwrap();
    }
}
#[test]
fn complete_body_matches_independent_workers_and_shared_immutable_readers() {
    let body = actual_tests::actual_body();
    let expected = body.replay_bytes();
    let workers: Vec<_> = (0..4).map(|_| std::thread::spawn(actual_receipt)).collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| getter_snapshot::capture(&body)))
            .collect();
        let getters = getter_snapshot::capture(&body);
        for reader in readers {
            assert_eq!(reader.join().unwrap(), getters);
        }
    });
}
#[test]
fn actual_body_preserves_caller_namespace_even_for_identical_geometry() {
    let first = default_resolved_prism_fixture();
    let body = project_named_prism_body(&first.publication, &first.names).unwrap();
    let mut request = first.request.clone();
    request.operation.0 = u64::MAX;
    request.sources[0].source.0 = 2056;
    let second = resolved_prism_fixture(request);
    let named = project_named_prism_body(&second.publication, &second.names).unwrap();
    assert_eq!(
        first.publication.replay_bytes(),
        second.publication.replay_bytes()
    );
    assert_ne!(body.replay_bytes(), named.replay_bytes());
    assert_eq!(named.root().0[0].member.operation.0, u64::MAX);
    assert_eq!(named.root().0[0].member.source.0, 2056);
    assert!(!body.binds(&second.publication, &second.names));
}
#[test]
fn source_pole_and_traversal_reversals_keep_complete_semantic_body_views() {
    let base = default_resolved_prism_fixture().request;
    for (poles, traversal) in [(false, true), (true, false), (true, true)] {
        let mut request = base.clone();
        request.sources[0].reversed_traversal = traversal;
        if poles {
            request.sources[0].world_poles.reverse();
        }
        let fixture = resolved_prism_fixture(request);
        let body = project_named_prism_body(&fixture.publication, &fixture.names).unwrap();
        assert_eq!((body.members().len(), body.occurrences().len()), (13, 26));
        assert_eq!(
            body.faces().iter().map(|f| f.trims().len()).sum::<usize>(),
            6
        );
        assert_eq!(body.names().identities(), fixture.names.plan().identities());
    }
}
