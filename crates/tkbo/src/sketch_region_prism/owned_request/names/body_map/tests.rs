// SPDX-License-Identifier: GPL-2.0-only
use super::{super::*, test_fixture::*, types::*};
use crate::sketch_region_prism::frozen::{FrozenSweepRole, FrozenUseKey};
#[test]
fn literal_full_member_and_directed_domains_have_injective_semantic_expansion() {
    let names = names();
    assert_eq!(names.member_bindings().len(), 13);
    assert_eq!(names.use_bindings().len(), 26);
    assert_eq!(names.version(), 1);
    assert_eq!(names.plan(), &plan());
    for row in members() {
        assert_eq!(names.member(&row.frozen).unwrap(), &row.semantic);
    }
    for row in uses() {
        assert_eq!(names.occurrence(&row.frozen).unwrap(), &row.semantic);
    }
}
#[test]
fn wire_members_retain_source_role_kind_without_invented_output_tokens() {
    let names = names();
    let wires: Vec<_> = names
        .member_bindings()
        .iter()
        .filter(|row| row.semantic.kind == 5)
        .collect();
    assert_eq!(wires.len(), 3);
    assert!(wires.iter().all(|row| row.output_slot.is_none()));
    assert_ne!(wires[0].semantic, wires[1].semantic);
    assert_ne!(wires[1].semantic, wires[2].semantic);
    assert!(
        wires
            .iter()
            .all(|row| row.semantic.source_member.role != SemanticNameRole::LoopUse)
    );
}
#[test]
fn repeated_generic_use_slots_do_not_collapse_cap_side_or_seam_occurrences() {
    let names = names();
    let loops: Vec<_> = names
        .use_bindings()
        .iter()
        .filter(|row| row.semantic.0.last().unwrap().directed_use.role == SemanticNameRole::LoopUse)
        .collect();
    assert_eq!(loops.len(), 3);
    assert!(loops.iter().all(|row| row.semantic.0.len() == 4));
    let seams = seam_rows(&names);
    assert_eq!(seams.len(), 2);
    assert_eq!(
        seams[0].semantic.0.last().unwrap().member,
        seams[1].semantic.0.last().unwrap().member
    );
    assert_ne!(seams[0].semantic, seams[1].semantic);
}
#[test]
fn duplicate_plus_omission_and_extra_rows_refuse_exact_coverage() {
    let mut duplicate = members();
    duplicate[1] = duplicate[0].clone();
    assert!(matches!(
        checked(duplicate, uses()).unwrap_err().cause,
        PrismSemanticMapCause::MemberMultiplicity { .. }
    ));
    let mut missing = uses();
    missing.pop();
    assert!(matches!(
        checked(members(), missing).unwrap_err().cause,
        PrismSemanticMapCause::UseMultiplicity { .. }
    ));
    let mut extra = members();
    extra.push(extra[0].clone());
    assert!(checked(extra, uses()).is_err());
}
#[test]
fn caller_namespace_and_output_metadata_cannot_be_replaced() {
    let mut wrong = members();
    wrong[0].semantic.operation = CallerOperationId(0);
    assert!(matches!(
        checked(wrong, uses()).unwrap_err().cause,
        PrismSemanticMapCause::MemberExpansion { .. }
    ));
    let mut wire = members();
    wire[5].output_slot = Some(SemanticNameSlot {
        stage: SemanticNameStage::Output,
        role: SemanticNameRole::LoopUse,
    });
    assert!(matches!(
        checked(wire, uses()).unwrap_err().cause,
        PrismSemanticMapCause::OutputSlot { .. }
    ));
}
#[test]
fn mapping_permutations_have_identical_complete_replay() {
    let expected = names();
    let mut members = members();
    members.reverse();
    let mut uses = uses();
    uses.rotate_left(11);
    let actual = checked(members, uses).unwrap();
    assert_eq!(actual.replay_bytes(), expected.replay_bytes());
    assert!(
        actual
            .replay_bytes()
            .windows(b"literal-symbolic-domain-only".len())
            .any(|bytes| bytes == b"literal-symbolic-domain-only")
    );
}
#[test]
fn missing_lookup_preserves_typed_portable_key() {
    let names = names();
    let missing = key(999, FrozenSweepRole::First, 4);
    assert!(
        matches!(names.member(&missing).unwrap_err().cause,PrismSemanticMapCause::MemberMultiplicity{key,..} if key==missing)
    );
    assert!(matches!(
        names.occurrence(&FrozenUseKey(vec![])).unwrap_err().cause,
        PrismSemanticMapCause::UseMultiplicity { .. }
    ));
}
#[test]
fn owned_map_and_keys_are_send_sync_and_read_without_mutable_state() {
    fn immutable<T: Send + Sync>() {}
    immutable::<ResolvedPrismNames>();
    immutable::<SemanticMemberKey>();
    immutable::<SemanticUseKey>();
    immutable::<PrismSemanticMapFailure>();
    immutable::<PrismSemanticMapCause>();
    let expected = names().replay_bytes().to_vec();
    let worker = std::thread::spawn(|| names().replay_bytes().to_vec());
    assert_eq!(worker.join().unwrap(), expected);
}

fn seam_rows(names: &ResolvedPrismNames) -> Vec<&SemanticUseBinding> {
    names
        .use_bindings()
        .iter()
        .filter(|row| {
            matches!(
                row.semantic.0.last().unwrap().directed_use.role,
                SemanticNameRole::SeamStartUse | SemanticNameRole::SeamEndUse
            )
        })
        .collect()
}

#[test]
fn semantic_map_replay_probe() {
    let names = names();
    println!("SEMANTIC_MAP_REPLAY={:?}", names.replay_bytes());
    assert_eq!(names.member_bindings().len(), 13);
}
