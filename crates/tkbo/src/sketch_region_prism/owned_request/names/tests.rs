// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn ids() -> SingleBoundaryIdentities {
    SingleBoundaryIdentities {
        operation: CallerOperationId(42),
        boundary: CallerBoundaryId(700),
        closure: CallerVertexId(201),
        start: CallerDirectedUseId(990),
        end: CallerDirectedUseId(3),
    }
}
fn intent() -> SemanticSelectionIntent {
    SemanticSelectionIntent::BoundedRegionContaining { seed: [-0., 1.25] }
}
fn plan() -> SingleBoundaryNamePlan {
    SingleBoundaryNamePlan::v1(ids(), intent()).unwrap()
}
#[test]
fn complete_dictionary_has_fixed_unique_tokens_and_exact_reverse_projection() {
    let plan = plan();
    assert_eq!(plan.entries().len(), 97);
    for pair in plan.entries().windows(2) {
        assert!(pair[0].token < pair[1].token);
    }
    for entry in plan.entries() {
        assert_eq!(plan.reverse(entry.token.value()).unwrap(), entry);
    }
    assert_eq!(
        plan.reverse(0),
        Err(SemanticNameFailure::UnknownToken {
            actual: 0,
            version: 1
        })
    );
    assert_eq!(plan.identities(), ids());
    assert_eq!(plan.selection(), intent());
}
#[test]
fn caller_values_equal_to_internal_tokens_never_alias_namespaces() {
    let caller = SingleBoundaryIdentities {
        operation: CallerOperationId(257),
        boundary: CallerBoundaryId(515),
        closure: CallerVertexId(516),
        start: CallerDirectedUseId(517),
        end: CallerDirectedUseId(518),
    };
    let plan = SingleBoundaryNamePlan::v1(caller, intent()).unwrap();
    let expected = [
        (257, SemanticCallerAnchor::Operation(caller.operation)),
        (515, SemanticCallerAnchor::Boundary(caller.boundary)),
        (516, SemanticCallerAnchor::Closure(caller.closure)),
        (517, SemanticCallerAnchor::Start(caller.start)),
        (518, SemanticCallerAnchor::End(caller.end)),
    ];
    for (token, anchor) in expected {
        assert_eq!(plan.reverse(token).unwrap().anchor, anchor);
    }
}
#[test]
fn full_caller_range_is_available_without_internal_token_reservation() {
    let mut caller = ids();
    caller.operation = CallerOperationId(0);
    caller.boundary = CallerBoundaryId(u64::MAX);
    caller.start = CallerDirectedUseId(0);
    caller.end = CallerDirectedUseId(u64::MAX);
    assert_eq!(
        SingleBoundaryNamePlan::v1(caller, intent())
            .unwrap()
            .identities(),
        caller
    );
}
#[test]
fn swapped_endpoint_names_are_not_sorted_into_the_same_request() {
    let mut swapped = ids();
    std::mem::swap(&mut swapped.start, &mut swapped.end);
    let changed = SingleBoundaryNamePlan::v1(swapped, intent()).unwrap();
    assert_ne!(changed, plan());
    assert_eq!(
        changed.reverse(517).unwrap().anchor,
        SemanticCallerAnchor::Start(ids().end)
    );
    assert_eq!(
        changed.reverse(518).unwrap().anchor,
        SemanticCallerAnchor::End(ids().start)
    );
}
#[test]
fn request_rejects_equal_uses_and_nonfinite_selection_with_exact_values() {
    let mut caller = ids();
    caller.end = caller.start;
    assert_eq!(
        SingleBoundaryNamePlan::v1(caller, intent()),
        Err(SemanticNameFailure::EqualEndpointUses {
            actual: caller.start
        })
    );
    for seed in [[f64::INFINITY, 0.], [0., f64::NEG_INFINITY]] {
        let actual = SemanticSelectionIntent::BoundedRegionContaining { seed };
        assert!(matches!(SingleBoundaryNamePlan::v1(ids(), actual),
            Err(SemanticNameFailure::NonfiniteSeed { axis, actual }) if actual == seed[axis]));
    }
}
#[test]
fn nan_selection_is_rejected_before_a_dictionary_is_returned() {
    let selection = SemanticSelectionIntent::BoundedRegionContaining {
        seed: [0., f64::NAN],
    };
    assert!(matches!(SingleBoundaryNamePlan::v1(ids(), selection),
        Err(SemanticNameFailure::NonfiniteSeed { axis: 1, actual }) if actual.is_nan()));
}
#[test]
fn full_envelope_matches_independently_encoded_literal_fixture() {
    let envelope = SemanticReplayEnvelope::new(plan(), &[0, 255, 42]);
    assert_eq!(
        envelope.replay_bytes(),
        include_bytes!("fixtures/names-v1.bin")
    );
    assert_eq!(envelope.child_bytes(), [0, 255, 42]);
    assert_eq!(envelope.plan(), &plan());
    assert!(envelope.matches_plan(&plan()));
}
#[test]
fn operation_and_seed_bits_bind_even_when_child_bytes_are_identical() {
    let envelope = SemanticReplayEnvelope::new(plan(), &[7]);
    let mut other = ids();
    other.operation = CallerOperationId(43);
    let changed = SingleBoundaryNamePlan::v1(other, intent()).unwrap();
    assert!(!envelope.matches_plan(&changed));
    assert_ne!(
        envelope.replay_bytes(),
        SemanticReplayEnvelope::new(changed, &[7]).replay_bytes()
    );
    let positive_zero = SemanticSelectionIntent::BoundedRegionContaining { seed: [0., 1.25] };
    assert!(!envelope.matches_plan(&SingleBoundaryNamePlan::v1(ids(), positive_zero).unwrap()));
}
#[test]
fn child_mutation_cannot_change_an_owned_envelope_or_its_name_binding() {
    let mut child = vec![1, 2, 3];
    let envelope = SemanticReplayEnvelope::new(plan(), &child);
    let before = envelope.replay_bytes().to_vec();
    child[0] = 99;
    assert_eq!(envelope.child_bytes(), [1, 2, 3]);
    assert_eq!(envelope.replay_bytes(), before);
    assert_ne!(envelope, SemanticReplayEnvelope::new(plan(), &child));
}
#[test]
fn legacy_containment_bytes_are_preserved_as_an_opaque_payload() {
    let legacy = include_bytes!("../../frozen/fixtures/containment-v1.bin");
    let envelope = SemanticReplayEnvelope::new(plan(), legacy);
    assert_eq!(envelope.child_bytes(), legacy);
    assert!(envelope.replay_bytes().ends_with(legacy));
}
#[test]
fn owned_names_and_envelope_are_send_sync_and_thread_independent() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<SingleBoundaryIdentities>();
    send_sync::<SemanticNameEntry>();
    send_sync::<SemanticNameFailure>();
    send_sync::<SingleBoundaryNamePlan>();
    send_sync::<SemanticReplayEnvelope>();
    let expected = SemanticReplayEnvelope::new(plan(), &[0, 255, 42]);
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| SemanticReplayEnvelope::new(plan(), &[0, 255, 42])))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
}
#[test]
fn semantic_replay_probe() {
    let envelope = SemanticReplayEnvelope::new(plan(), &[0, 255, 42]);
    println!("SEMANTIC_REPLAY={:?}", envelope.replay_bytes());
    assert_eq!(
        envelope.replay_bytes(),
        include_bytes!("fixtures/names-v1.bin")
    );
}

#[test]
fn every_caller_identity_changes_full_binding_without_changing_fixed_tokens() {
    let baseline = SemanticReplayEnvelope::new(plan(), &[1]);
    for caller in changed_identities() {
        let changed = SingleBoundaryNamePlan::v1(caller, intent()).unwrap();
        assert!(!baseline.matches_plan(&changed));
        assert_ne!(
            baseline.replay_bytes(),
            SemanticReplayEnvelope::new(changed, &[1]).replay_bytes()
        );
    }
}
fn changed_identities() -> [SingleBoundaryIdentities; 5] {
    let mut values = [ids(); 5];
    values[0].operation.0 += 1;
    values[1].boundary.0 += 1;
    values[2].closure.0 += 1;
    values[3].start.0 += 1;
    values[4].end.0 += 1;
    values
}
#[test]
fn every_scalar_name_type_has_an_owned_send_sync_contract() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<CallerOperationId>();
    send_sync::<CallerBoundaryId>();
    send_sync::<CallerVertexId>();
    send_sync::<CallerDirectedUseId>();
    send_sync::<SemanticSelectionIntent>();
    send_sync::<SemanticCallerAnchor>();
    send_sync::<SemanticNameStage>();
    send_sync::<SemanticNameRole>();
    send_sync::<SemanticNameSlot>();
    send_sync::<SemanticNameToken>();
}
