// SPDX-License-Identifier: GPL-2.0-only
//! Deliberately edited symbolic domains test exact table admission, not modelling acceptance.
use super::{super::*, checks, test_fixture::*, types::*};
use crate::sketch_region_prism::frozen::AuthoredPrismId;
fn edited_domain(
    plan: SingleBoundaryNamePlan,
    members: Vec<SemanticMemberBinding>,
    uses: Vec<SemanticUseBinding>,
    bytes: &[u8],
) -> MapCheck<ResolvedPrismNames> {
    let actual_members: Vec<_> = members.iter().map(|row| row.frozen).collect();
    let actual_uses: Vec<_> = uses.iter().map(|row| row.frozen.clone()).collect();
    checks::checked(
        plan,
        NameMapDomain {
            members: &actual_members,
            uses: &actual_uses,
            publication: bytes,
        },
        members,
        uses,
    )
}
#[test]
fn duplicated_domain_and_table_cannot_agree_their_way_into_unique_coverage() {
    let mut rows = members();
    rows.push(rows[0].clone());
    let error = edited_domain(plan(), rows, uses(), b"duplicate").unwrap_err();
    assert!(matches!(
        error.cause,
        PrismSemanticMapCause::MemberMultiplicity {
            actual: 2,
            domain: 2,
            expected: 1,
            ..
        }
    ));
}
#[test]
fn missing_intermediate_ancestry_refuses_even_if_domain_and_rows_omit_it() {
    let mut rows = uses();
    rows.retain(|row| row.frozen.0.len() != 2);
    let error = edited_domain(plan(), members(), rows, b"ancestor").unwrap_err();
    assert!(matches!(
        error.cause,
        PrismSemanticMapCause::MissingAncestor { .. }
    ));
}
#[test]
fn unknown_source_token_and_wrong_operation_keep_exact_offending_ids() {
    let mut unknown = members();
    unknown[0].frozen.source = AuthoredPrismId(999);
    let error = edited_domain(plan(), unknown, uses(), b"unknown").unwrap_err();
    assert_eq!(
        error.cause,
        PrismSemanticMapCause::UnknownToken { actual: 999 }
    );
    let mut wrong = members();
    wrong[0].frozen.operation = AuthoredPrismId(17);
    let error = edited_domain(plan(), wrong, uses(), b"operation").unwrap_err();
    assert_eq!(
        error.cause,
        PrismSemanticMapCause::Operation {
            actual: 17,
            expected: 257
        }
    );
}
#[test]
fn valid_dictionary_use_token_is_not_a_member_source_name() {
    let mut rows = members();
    rows[0].frozen.source = AuthoredPrismId(2333);
    let error = edited_domain(plan(), rows, uses(), b"role").unwrap_err();
    assert!(matches!(
        error.cause,
        PrismSemanticMapCause::UnsupportedSourceSlot {
            slot: SemanticNameSlot {
                stage: SemanticNameStage::Output,
                role: SemanticNameRole::LoopUse
            }
        }
    ));
}
#[test]
fn valid_face_use_token_cannot_name_wire_occurrence() {
    let mut rows = uses();
    let row = rows.iter_mut().find(|row| row.frozen.0.len() == 4).unwrap();
    row.frozen.0.last_mut().unwrap().1 = AuthoredPrismId(2332);
    row.semantic.0.last_mut().unwrap().directed_use.role = SemanticNameRole::FaceUse;
    let error = edited_domain(plan(), members(), rows, b"use").unwrap_err();
    assert!(matches!(
        error.cause,
        PrismSemanticMapCause::UnsupportedUseSlot { kind: 5, .. }
    ));
}
#[test]
fn caller_identity_changes_full_map_even_when_internal_tokens_and_child_match() {
    let original = edited_domain(plan(), members(), uses(), b"same").unwrap();
    let mut ids = plan().identities();
    ids.operation = CallerOperationId(0);
    ids.boundary = CallerBoundaryId(257);
    let changed = SingleBoundaryNamePlan::v1(ids, plan().selection()).unwrap();
    let (member_rows, use_rows) = caller_rows(ids);
    let rebound = edited_domain(changed, member_rows, use_rows, b"same").unwrap();
    assert_ne!(original.replay_bytes(), rebound.replay_bytes());
    assert_ne!(
        original.member_bindings()[0].semantic,
        rebound.member_bindings()[0].semantic
    );
}
#[test]
fn exact_seed_bits_and_complete_child_bytes_bind_map_replay() {
    let original = edited_domain(plan(), members(), uses(), b"same").unwrap();
    let positive = SingleBoundaryNamePlan::v1(
        plan().identities(),
        SemanticSelectionIntent::BoundedRegionContaining { seed: [0., 1.25] },
    )
    .unwrap();
    let seed = edited_domain(positive, members(), uses(), b"same").unwrap();
    let child = edited_domain(plan(), members(), uses(), b"samE").unwrap();
    assert_ne!(original.replay_bytes(), seed.replay_bytes());
    assert_ne!(original.replay_bytes(), child.replay_bytes());
}
#[test]
fn empty_domains_and_empty_use_paths_are_typed_incomplete_inputs() {
    let error = edited_domain(plan(), vec![], vec![], b"").unwrap_err();
    assert_eq!(
        error.cause,
        PrismSemanticMapCause::EmptyDomain {
            members: 0,
            uses: 0
        }
    );
    let mut rows = uses();
    rows[0].frozen.0.clear();
    rows[0].semantic.0.clear();
    let error = edited_domain(plan(), members(), rows, b"empty-use").unwrap_err();
    assert_eq!(error.cause, PrismSemanticMapCause::EmptyUse);
}
#[test]
fn caller_table_edits_cannot_mutate_retained_map() {
    let mut rows = members();
    let map = checked(rows.clone(), uses()).unwrap();
    let before = map.replay_bytes().to_vec();
    rows[0].semantic.source = CallerBoundaryId(99);
    assert_eq!(map.replay_bytes(), before);
    assert_ne!(
        rows[0].semantic,
        map.member(&rows[0].frozen).unwrap().clone()
    );
}

fn caller_rows(
    ids: SingleBoundaryIdentities,
) -> (Vec<SemanticMemberBinding>, Vec<SemanticUseBinding>) {
    let mut member_rows = members();
    let mut use_rows = uses();
    for row in &mut member_rows {
        row.semantic.operation = ids.operation;
        row.semantic.source = ids.boundary;
    }
    for row in &mut use_rows {
        for step in &mut row.semantic.0 {
            step.member.operation = ids.operation;
            step.member.source = ids.boundary;
        }
    }
    (member_rows, use_rows)
}
#[test]
fn supported_dictionary_source_with_wrong_body_kind_remains_unsupported() {
    let mut rows = members();
    rows[0].frozen.kind = 1;
    let error = edited_domain(plan(), rows, uses(), b"unsupported-kind").unwrap_err();
    assert!(matches!(error.cause,PrismSemanticMapCause::UnsupportedMember {key} if key.kind==1));
}
