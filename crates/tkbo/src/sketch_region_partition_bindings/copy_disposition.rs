// SPDX-License-Identifier: GPL-2.0-only
//! An unrepresented copy is an explicit disposition, never an input deletion.
use super::{history_parents::*, history_types::*, occurrence_keys::same_physical_occurrence};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_candidates::CandidateCopyInspection;

pub(super) fn inspect_copy_dispositions(
    copy: &CandidateCopyInspection,
    inspection: &mut CandidateHistoryInspection,
) {
    super::copied_tree_bindings::inspect_copied_tree(copy, inspection);
    let Ok(batch) = &copy.completion else {
        return;
    };
    for (index, link) in copy.links.iter().enumerate() {
        let unrepresented = unrepresented_rows(batch, &link.copied);
        inspect_copy_row(index, copy, unrepresented, inspection);
    }
    inspect_unrepresented(batch, &copy.links, inspection);
    inspect_duplicate_copies(copy, inspection);
}
fn unrepresented_rows(
    batch: &crate::sketch_region_candidates::CandidateBatch,
    key: &PartitionOccurrenceIdentity,
) -> Vec<usize> {
    batch
        .unrepresented
        .iter()
        .enumerate()
        .filter(|(_, relation)| {
            same_physical_occurrence(key, &PartitionOccurrenceIdentity::capture(&relation.output))
        })
        .map(|(row, _)| row)
        .collect()
}
fn represented_rows(
    key: &PartitionOccurrenceIdentity,
    inspection: &CandidateHistoryInspection,
) -> Vec<(usize, usize)> {
    inspection
        .occurrences
        .iter()
        .enumerate()
        .filter(|(_, row)| same_physical_occurrence(key, &row.key))
        .map(|(row, occurrence)| (occurrence.candidate, row))
        .collect()
}
fn inspect_copy_row(
    index: usize,
    copy: &CandidateCopyInspection,
    unrepresented: Vec<usize>,
    inspection: &mut CandidateHistoryInspection,
) {
    let link = &copy.links[index];
    let Some(supported) = observed_history_supported(link, inspection) else {
        return;
    };
    let row = CopyRowDisposition {
        copy_row: index,
        represented: represented_rows(&link.copied, inspection),
        unrepresented,
        skipped_unsupported_kind: !supported,
    };
    record_disposition(row, &link.copied, inspection);
}
fn observed_history_supported(
    link: &crate::sketch_region_candidates::CandidateImageCopyLink,
    inspection: &mut CandidateHistoryInspection,
) -> Option<bool> {
    match &link.history_supported {
        Ok(supported) => Some(*supported),
        Err(cause) => {
            copy_fault(
                &link.copied,
                CandidateHistoryCause::Access(cause.clone()),
                inspection,
            );
            None
        }
    }
}
fn disposition_fault(row: &CopyRowDisposition) -> Option<CandidateHistoryCause> {
    if row.skipped_unsupported_kind {
        return None;
    }
    if row.represented.is_empty() && row.unrepresented.is_empty() {
        return Some(CandidateHistoryCause::MissingDisposition {
            copy_row: row.copy_row,
        });
    }
    if !row.represented.is_empty() && !row.unrepresented.is_empty() {
        return Some(CandidateHistoryCause::ConflictingDisposition {
            copy_row: row.copy_row,
        });
    }
    None
}

fn inspect_unrepresented(
    batch: &crate::sketch_region_candidates::CandidateBatch,
    links: &[crate::sketch_region_candidates::CandidateImageCopyLink],
    inspection: &mut CandidateHistoryInspection,
) {
    for (row, relation) in batch.unrepresented.iter().enumerate() {
        inspect_unrepresented_row(row, relation, links, inspection);
    }
}
fn inspect_unrepresented_row(
    row: usize,
    relation: &crate::sketch_region::ArrangementRelation,
    links: &[crate::sketch_region_candidates::CandidateImageCopyLink],
    inspection: &mut CandidateHistoryInspection,
) {
    let key = PartitionOccurrenceIdentity::capture(&relation.output);
    let copies = physical_copy_rows(&key, links);
    if copies.is_empty() {
        copy_fault(
            &key,
            CandidateHistoryCause::MissingUnrepresentedCopy { row },
            inspection,
        );
        return;
    }
    for cause in history_parent_faults(&composed_parents(&copies, links), relation) {
        copy_fault(&key, cause, inspection);
    }
}

fn inspect_duplicate_copies(
    copy: &CandidateCopyInspection,
    inspection: &mut CandidateHistoryInspection,
) {
    for (repeated, link) in copy.links.iter().enumerate() {
        let first = copy.links[..repeated]
            .iter()
            .position(|earlier| earlier.original == link.original && earlier.copied == link.copied);
        if let Some(first) = first {
            copy_fault(
                &link.copied,
                CandidateHistoryCause::DuplicateDirectedCopy { first, repeated },
                inspection,
            );
        }
    }
}
fn copy_fault(
    key: &PartitionOccurrenceIdentity,
    cause: CandidateHistoryCause,
    inspection: &mut CandidateHistoryInspection,
) {
    inspection.faults.push(CandidateHistoryFault {
        candidate: None,
        key: key.clone(),
        cause,
    });
}

fn record_disposition(
    row: CopyRowDisposition,
    key: &PartitionOccurrenceIdentity,
    inspection: &mut CandidateHistoryInspection,
) {
    if let Some(cause) = disposition_fault(&row) {
        copy_fault(key, cause, inspection);
    }
    inspection.copy_dispositions.push(row);
}
