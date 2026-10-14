// SPDX-License-Identifier: GPL-2.0-only
//! Adversarial authored rosters refuse; no subset, count or reused name grants success.
use super::fixture::{AnnularHistoryCase, copy_scope};
use super::operation_tests::inspected;
use super::*;
use crate::sketch_region::RegionSourceId;

pub(super) struct Refusal {
    pub class: PrismHistoryFailureClass,
    pub phase: PrismHistoryPhase,
    pub cause: String,
    pub join: bool,
}
pub(super) fn refused(case: &AnnularHistoryCase) -> Refusal {
    case.run(
        |request| match inspect_annular_prism_history(request, |_| false) {
            AnnularHistoryOutcome::Incomplete(f) | AnnularHistoryOutcome::Failed(f) => Refusal {
                class: f.finding().class,
                phase: f.finding().phase,
                cause: format!("{:?}", f.finding().cause),
                join: f.join.is_some(),
            },
            AnnularHistoryOutcome::Inspected(_) => panic!("expected refusal"),
            AnnularHistoryOutcome::Cancelled { .. } => panic!("unexpected cancellation"),
        },
    )
}
pub(super) fn assert_refused(case: &AnnularHistoryCase, required: &str) {
    let refusal = refused(case);
    let cause = &refusal.cause;
    assert!(cause.contains(required), "{required:?} not in {cause}");
    assert_eq!(refusal.class, PrismHistoryFailureClass::Incomplete);
}
fn scopes(case: &mut AnnularHistoryCase) -> &mut Vec<AnnularScopeBindings> {
    &mut case.names.historical.scopes
}
fn index(case: &AnnularHistoryCase, scope: AnnularHistoryScope) -> usize {
    let scopes = &case.names.historical.scopes;
    scopes.iter().position(|s| s.scope == scope).unwrap()
}
#[test]
fn missing_and_duplicate_scopes_refuse_at_admission() {
    let mut case = AnnularHistoryCase::nested();
    let at = index(&case, AnnularHistoryScope::Partition);
    let partition = scopes(&mut case).remove(at);
    assert_refused(&case, "exact authored intermediate scope roster");
    let duplicate = copy_scope(&partition);
    scopes(&mut case).push(partition);
    scopes(&mut case).push(duplicate);
    assert_refused(&case, "no duplicate authored history scope");
    scopes(&mut case).pop();
    inspected(&case);
}
#[test]
fn extra_and_mislabeled_scopes_refuse_at_admission() {
    let mut case = AnnularHistoryCase::nested();
    let mut extra = copy_scope(&case.names.historical.scopes[0]);
    extra.scope = AnnularHistoryScope::Rejected(999);
    scopes(&mut case).push(extra);
    assert_refused(&case, "exact authored intermediate scope roster");
    scopes(&mut case).pop();
    let at = index(&case, AnnularHistoryScope::Partition);
    scopes(&mut case)[at].names.stage = PrismHistoryStage::InputCopy;
    assert_refused(&case, "authored scope stage matches scope");
}
#[test]
fn unused_extra_use_names_refuse_without_implicit_identity() {
    let mut case = AnnularHistoryCase::nested();
    let at = index(&case, AnnularHistoryScope::Partition);
    let names = &mut scopes(&mut case)[at].names;
    let actual = names.uses[0].actual.clone();
    let extra = PrismHistoricalUseBinding {
        actual,
        authored_use: AuthoredPrismId(9999),
    };
    names.uses.push(extra);
    assert_refused(&case, "one authored binding per historical use");
}
#[test]
fn missing_and_conflicting_member_names_refuse() {
    let mut case = AnnularHistoryCase::nested();
    let at = index(&case, AnnularHistoryScope::Partition);
    let member = scopes(&mut case)[at].names.members.pop().unwrap();
    assert_refused(&case, "exactly one authored historical definition");
    let duplicate = PrismSourceBinding {
        actual: member.actual.clone(),
        member: AuthoredPrismId(9998),
    };
    let names = &mut scopes(&mut case)[at].names;
    names.members.push(member);
    names.members.push(duplicate);
    assert_refused(&case, "exactly one authored historical definition");
}
/// Reusing the outer copy's member names for the inner copy would merge distinct intermediates.
#[test]
fn reused_member_names_across_source_scopes_refuse_distinct_key_admission() {
    let mut case = AnnularHistoryCase::nested();
    let outer = index(&case, AnnularHistoryScope::InputCopy(RegionSourceId(101)));
    let inner = index(&case, AnnularHistoryScope::InputCopy(RegionSourceId(202)));
    let outer = &case.names.historical.scopes[outer].names.members;
    let mut names: Vec<_> = outer.iter().map(|m| m.member).collect();
    names.sort();
    let inner = scopes(&mut case)[inner].names.members.iter_mut();
    let mut inner: Vec<_> = inner.collect();
    inner.sort_by_key(|m| m.member);
    for (row, name) in inner.into_iter().zip(names) {
        row.member = name;
    }
    assert_refused(
        &case,
        "authored member keys are distinct across every scope",
    );
}
/// Same-parent closure uses stay distinct intermediates; their parent sets are equal by design.
#[test]
fn equal_parent_sets_keep_distinct_closure_use_intermediates() {
    let history = inspected(&AnnularHistoryCase::nested());
    let rows = history.partition_relations();
    let vertex = |r: &&PrismHistoricalRelation| r.parents[1].len() == 2;
    let vertices: Vec<_> = rows.iter().filter(vertex).collect();
    assert_eq!(vertices.len(), 4);
    let same = |a: &PrismHistoricalRelation, b: &PrismHistoricalRelation| a.parents == b.parents;
    let pairs = vertices
        .iter()
        .filter(|a| vertices.iter().any(|b| a.output != b.output && same(a, b)));
    assert_eq!(pairs.count(), 4);
}
#[test]
fn swapped_source_names_are_refused_by_join_rebind_and_history_reprojection() {
    let mut case = AnnularHistoryCase::nested();
    let history = inspected(&case);
    let originals = &mut case.publication.originals;
    let (source, member, directed) = (
        originals[0].actual.source,
        originals[0].member,
        originals[0].directed_use,
    );
    let other = |o: &PrismOriginalBinding| o.actual.source != source && o.directed_use == directed;
    let other = originals.iter().position(other).unwrap();
    originals[0].member = originals[other].member;
    originals[other].member = member;
    let refusal = refused(&case);
    assert!(
        refusal.join,
        "join rebind must refuse renamed sources: {}",
        refusal.cause
    );
    let reprojected = case.run(|r| reproject_annular_prism_history(r, &history).is_err());
    assert!(reprojected);
}
