// SPDX-License-Identifier: GPL-2.0-only
//! Negatives corrupt only owned history receipts, not mapped geometry or tolerance policy.
use super::{candidate_history::*, copy_tests::*, history_types::*};
use crate::sketch_region_candidates::{CandidateCopyInspection, composed_copy_relation};

fn actual_history_copy() -> CandidateCopyInspection {
    let (_, owned) = actual_copy_inputs();
    actual_candidate_copy(&owned, None)
}
#[test]
fn actual_directed_candidate_history_accounts_for_every_supported_copy() {
    let copy = actual_history_copy();
    let inspection = inspect_candidate_history(&copy);
    assert!(inspection.mapped_candidates_available);
    assert!(!inspection.occurrences.is_empty());
    assert_eq!(inspection.copy_dispositions.len(), copy.links.len());
    assert!(inspection.faults.is_empty());
}
#[test]
fn missing_directed_relation_is_not_recovered_from_equal_parent_sets() {
    let mut copy = actual_history_copy();
    copy.completion.as_mut().ok().unwrap().candidates[0]
        .relations
        .remove(0);
    let inspection = inspect_candidate_history(&copy);
    assert!(inspection.faults.iter().any(|fault| matches!(
        fault.cause,
        CandidateHistoryCause::RelationMultiplicity { count: 0 }
    )));
}
#[test]
fn duplicate_directed_relation_does_not_pass_a_first_match_lookup() {
    let mut copy = actual_history_copy();
    let candidate = &mut copy.completion.as_mut().ok().unwrap().candidates[0];
    let duplicate = copied_relation_row(&candidate.relations[0]);
    candidate.relations.push(duplicate);
    let inspection = inspect_candidate_history(&copy);
    assert!(inspection.faults.iter().any(|fault| matches!(
        fault.cause,
        CandidateHistoryCause::RelationMultiplicity { count: 2 }
    )));
}
fn copied_relation_row(
    row: &crate::sketch_region::ArrangementRelation,
) -> crate::sketch_region::ArrangementRelation {
    crate::sketch_region::ArrangementRelation {
        output: row.output.clone(),
        preserved: row.preserved.clone(),
        modified: row.modified.clone(),
        generated: row.generated.clone(),
    }
}
#[test]
fn changing_a_relation_kind_keeps_full_parent_identity_but_is_rejected() {
    let mut copy = actual_history_copy();
    let candidate = &mut copy.completion.as_mut().ok().unwrap().candidates[0];
    let relation = candidate
        .relations
        .iter_mut()
        .find(|row| !row.modified.is_empty())
        .unwrap();
    relation.generated.append(&mut relation.modified);
    let inspection = inspect_candidate_history(&copy);
    assert!(
        inspection
            .faults
            .iter()
            .any(|fault| matches!(fault.cause, CandidateHistoryCause::KindedParents { .. }))
    );
}
#[test]
fn repeated_identical_directed_copy_row_is_an_explicit_ambiguity() {
    let mut copy = actual_history_copy();
    copy.links.push(copy.links[0].clone());
    let inspection = inspect_candidate_history(&copy);
    assert!(inspection.faults.iter().any(|fault| matches!(
        fault.cause,
        CandidateHistoryCause::DuplicateDirectedCopy { .. }
    )));
}
#[test]
fn an_extra_unrelated_output_relation_cannot_be_hidden_by_history_parents() {
    let mut copy = actual_history_copy();
    let unrelated = crate::sketch_region_candidates::tests::rectangle(10., 10., 2., 2.).remove(0);
    let candidate = &mut copy.completion.as_mut().ok().unwrap().candidates[0];
    candidate
        .relations
        .push(composed_copy_relation(&candidate.relations[0], &unrelated));
    let inspection = inspect_candidate_history(&copy);
    assert!(
        inspection
            .faults
            .iter()
            .any(|fault| matches!(fault.cause, CandidateHistoryCause::ExtraRelation { .. }))
    );
}
