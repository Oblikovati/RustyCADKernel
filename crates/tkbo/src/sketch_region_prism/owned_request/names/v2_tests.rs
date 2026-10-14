// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn identities() -> SingleBoundaryIdentities {
    SingleBoundaryIdentities {
        operation: CallerOperationId(42),
        boundary: CallerBoundaryId(700),
        closure: CallerVertexId(201),
        start: CallerDirectedUseId(990),
        end: CallerDirectedUseId(3),
    }
}
fn selection() -> SemanticSelectionIntent {
    SemanticSelectionIntent::BoundedRegionContaining { seed: [-0., 1.25] }
}
fn plan() -> SingleBoundaryNamePlan {
    SingleBoundaryNamePlan::v2(identities(), selection()).unwrap()
}
#[test]
fn v2_reserves_only_four_literal_aggregate_slots_beyond_v1() {
    let legacy = SingleBoundaryNamePlan::v1(identities(), selection()).unwrap();
    let current = plan();
    assert_eq!((legacy.version(), current.version()), (1, 2));
    assert_eq!((legacy.entries().len(), current.entries().len()), (97, 101));
    for entry in legacy.entries() {
        assert_eq!(current.reverse(entry.token.value()).unwrap(), entry);
    }
    assert_eq!(
        additional_entries(&current, &legacy),
        [(1058, 4, 34), (1059, 4, 35), (1314, 5, 34), (1315, 5, 35)]
    );
}
fn additional_entries(
    current: &SingleBoundaryNamePlan,
    legacy: &SingleBoundaryNamePlan,
) -> Vec<(u64, u8, u8)> {
    current
        .entries()
        .iter()
        .filter(|entry| legacy.reverse(entry.token.value()).is_err())
        .map(|entry| {
            (
                entry.token.value(),
                entry.slot.stage as u8,
                entry.slot.role as u8,
            )
        })
        .collect()
}
#[test]
fn legacy_admission_refuses_aggregate_tokens() {
    let legacy = SingleBoundaryNamePlan::v1(identities(), selection()).unwrap();
    for token in [1058, 1059, 1314, 1315] {
        assert_eq!(
            legacy.reverse(token),
            Err(SemanticNameFailure::UnknownToken {
                actual: token,
                version: 1,
            })
        );
        assert_eq!(
            plan().reverse(token).unwrap().anchor,
            SemanticCallerAnchor::Boundary(CallerBoundaryId(700))
        );
    }
}
#[test]
fn v2_refuses_aggregate_tokens_in_unreserved_stages() {
    for token in [546, 803, 1570, 1827, 2082, 2339] {
        assert_eq!(
            plan().reverse(token),
            Err(SemanticNameFailure::UnknownToken {
                actual: token,
                version: 2,
            })
        );
    }
}
#[test]
fn complete_v2_envelope_matches_independent_literal_extension_and_keeps_v1() {
    let current = SemanticReplayEnvelope::new(plan(), &[0, 255, 42]);
    assert_eq!(
        current.replay_bytes(),
        include_bytes!("fixtures/names-v2.bin")
    );
    let legacy = SingleBoundaryNamePlan::v1(identities(), selection()).unwrap();
    let old = SemanticReplayEnvelope::new(legacy, &[0, 255, 42]);
    assert_eq!(old.replay_bytes(), include_bytes!("fixtures/names-v1.bin"));
    assert_ne!(old.replay_bytes(), current.replay_bytes());
    assert!(!old.matches_plan(&plan()));
}
#[test]
fn aggregate_numeric_tokens_are_allowed_caller_values_without_aliasing() {
    let mut ids = identities();
    ids.operation = CallerOperationId(1058);
    ids.boundary = CallerBoundaryId(1315);
    ids.start = CallerDirectedUseId(1059);
    ids.end = CallerDirectedUseId(1314);
    let names = SingleBoundaryNamePlan::v2(ids, selection()).unwrap();
    for token in [1058, 1059, 1314, 1315] {
        let entry = names.reverse(token).unwrap();
        assert_eq!(
            (entry.operation, entry.source),
            (ids.operation, ids.boundary)
        );
        assert_eq!(entry.anchor, SemanticCallerAnchor::Boundary(ids.boundary));
    }
    assert_eq!(names.identities(), ids);
}
#[test]
fn v2_retains_identical_name_admission_and_exact_seed_bits() {
    let mut ids = identities();
    ids.end = ids.start;
    assert_eq!(
        SingleBoundaryNamePlan::v2(ids, selection()),
        Err(SemanticNameFailure::EqualEndpointUses { actual: ids.start })
    );
    let nan = SemanticSelectionIntent::BoundedRegionContaining {
        seed: [f64::NAN, 0.],
    };
    assert!(matches!(SingleBoundaryNamePlan::v2(identities(), nan),
        Err(SemanticNameFailure::NonfiniteSeed { axis: 0, actual }) if actual.is_nan()));
    let zero = SemanticSelectionIntent::BoundedRegionContaining { seed: [0., 1.25] };
    assert_ne!(
        plan(),
        SingleBoundaryNamePlan::v2(identities(), zero).unwrap()
    );
}
#[test]
fn semantic_v2_replay_probe() {
    let result = SemanticReplayEnvelope::new(plan(), &[0, 255, 42]);
    let threads: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(|| SemanticReplayEnvelope::new(plan(), &[0, 255, 42])))
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), result);
    }
    assert_eq!(
        result.replay_bytes(),
        include_bytes!("fixtures/names-v2.bin")
    );
    println!("SEMANTIC_V2_REPLAY={:?}", result.replay_bytes());
}
