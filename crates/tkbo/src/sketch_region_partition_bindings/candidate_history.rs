// SPDX-License-Identifier: GPL-2.0-only
//! Match directed candidate occurrences before evaluating their kinded history.
use super::{history_parents::*, history_types::*};
use crate::sketch_region::{PartitionOccurrenceIdentity, equal_occurrences};
use crate::sketch_region_candidates::{
    CandidateCopyInspection, CandidateImageCopyLink, RegionCandidate,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

struct CandidateHistoryContext<'a> {
    index: usize,
    candidate: &'a RegionCandidate,
    links: &'a [CandidateImageCopyLink],
}
pub(super) fn inspect_candidate_history(
    copy: &CandidateCopyInspection,
) -> CandidateHistoryInspection {
    let mut inspection = empty_history(copy.completion.is_ok());
    let Ok(batch) = &copy.completion else {
        return inspection;
    };
    for (index, candidate) in batch.candidates.iter().enumerate() {
        inspect_candidate(index, candidate, &copy.links, &mut inspection);
    }
    super::copy_disposition::inspect_copy_dispositions(copy, &mut inspection);
    inspection
}
fn empty_history(available: bool) -> CandidateHistoryInspection {
    CandidateHistoryInspection {
        mapped_candidates_available: available,
        occurrences: Vec::new(),
        copy_dispositions: Vec::new(),
        faults: Vec::new(),
    }
}
fn inspect_candidate(
    index: usize,
    candidate: &RegionCandidate,
    links: &[CandidateImageCopyLink],
    inspection: &mut CandidateHistoryInspection,
) {
    let Some(members) = candidate_members(index, candidate, inspection) else {
        return;
    };
    let context = CandidateHistoryContext {
        index,
        candidate,
        links,
    };
    for member in &members {
        inspect_member(member, &context, inspection);
    }
    inspect_extra_relations(&members, &context, inspection);
}
fn inspect_member(
    member: &Shape,
    context: &CandidateHistoryContext<'_>,
    inspection: &mut CandidateHistoryInspection,
) {
    let Some((copy_rows, parents)) = member_history(member, context, inspection) else {
        return;
    };
    let key = PartitionOccurrenceIdentity::capture(member);
    let relation_rows = directed_relations(&key, context.candidate);
    let row = CandidateHistoryOccurrence {
        candidate: context.index,
        key,
        copy_rows,
        relation_rows,
        parents,
    };
    inspect_relation(&row, context, inspection);
    inspection.occurrences.push(row);
}
fn member_history(
    member: &Shape,
    context: &CandidateHistoryContext<'_>,
    inspection: &mut CandidateHistoryInspection,
) -> Option<(Vec<usize>, KindedBindingParents)> {
    match expected_history(member, context.links) {
        Ok(expected) => Some(expected),
        Err(cause) => {
            access_fault(context.index, member, cause, inspection);
            None
        }
    }
}

fn directed_relations(
    key: &PartitionOccurrenceIdentity,
    candidate: &RegionCandidate,
) -> Vec<usize> {
    candidate
        .relations
        .iter()
        .enumerate()
        .filter(|(_, relation)| PartitionOccurrenceIdentity::capture(&relation.output) == *key)
        .map(|(row, _)| row)
        .collect()
}
fn inspect_relation(
    row: &CandidateHistoryOccurrence,
    context: &CandidateHistoryContext<'_>,
    inspection: &mut CandidateHistoryInspection,
) {
    inspect_required_copy(row, inspection);
    if row.relation_rows.len() != 1 {
        relation_multiplicity_fault(row, inspection);
        return;
    }
    let actual = &context.candidate.relations[row.relation_rows[0]];
    for cause in history_parent_faults(&row.parents, actual) {
        push_fault(context.index, &row.key, cause, inspection);
    }
}
fn relation_multiplicity_fault(
    row: &CandidateHistoryOccurrence,
    inspection: &mut CandidateHistoryInspection,
) {
    push_fault(
        row.candidate,
        &row.key,
        CandidateHistoryCause::RelationMultiplicity {
            count: row.relation_rows.len(),
        },
        inspection,
    );
}

fn inspect_extra_relations(
    members: &[Shape],
    context: &CandidateHistoryContext<'_>,
    inspection: &mut CandidateHistoryInspection,
) {
    let keys: Vec<_> = members
        .iter()
        .map(PartitionOccurrenceIdentity::capture)
        .collect();
    for (row, relation) in context.candidate.relations.iter().enumerate() {
        let key = PartitionOccurrenceIdentity::capture(&relation.output);
        if keys.contains(&key) {
            continue;
        }
        extra_relation_fault(context.index, row, &key, inspection);
    }
}

fn push_fault(
    index: usize,
    key: &PartitionOccurrenceIdentity,
    cause: CandidateHistoryCause,
    inspection: &mut CandidateHistoryInspection,
) {
    inspection.faults.push(CandidateHistoryFault {
        candidate: Some(index),
        key: key.clone(),
        cause,
    });
}
fn access_fault(
    index: usize,
    member: &Shape,
    cause: Failure,
    inspection: &mut CandidateHistoryInspection,
) {
    push_fault(
        index,
        &PartitionOccurrenceIdentity::capture(member),
        CandidateHistoryCause::Access(std::sync::Arc::new(cause)),
        inspection,
    );
}

fn candidate_members(
    index: usize,
    candidate: &RegionCandidate,
    inspection: &mut CandidateHistoryInspection,
) -> Option<Vec<Shape>> {
    match equal_occurrences(&candidate.face) {
        Ok(members) => Some(members),
        Err(cause) => {
            access_fault(index, &candidate.face, cause, inspection);
            None
        }
    }
}
fn inspect_required_copy(
    row: &CandidateHistoryOccurrence,
    inspection: &mut CandidateHistoryInspection,
) {
    if row.copy_rows.is_empty() {
        push_fault(
            row.candidate,
            &row.key,
            CandidateHistoryCause::MissingCopiedLeaf,
            inspection,
        );
    }
}
fn extra_relation_fault(
    index: usize,
    row: usize,
    key: &PartitionOccurrenceIdentity,
    inspection: &mut CandidateHistoryInspection,
) {
    push_fault(
        index,
        key,
        CandidateHistoryCause::ExtraRelation { row },
        inspection,
    );
}
