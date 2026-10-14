// SPDX-License-Identifier: GPL-2.0-only
//! Copied indices come from the actual copier graph, not a reconstructed result graph.
use super::{history_types::*, occurrence_keys::same_physical_occurrence};
use crate::sketch_region_candidates::{CandidateCopyInspection, CandidateCopyTreeObservation};

pub(super) fn inspect_copied_tree(
    copy: &CandidateCopyInspection,
    inspection: &mut CandidateHistoryInspection,
) {
    let members = match &copy.copied_tree {
        CandidateCopyTreeObservation::Captured(members) => members,
        _ => return,
    };
    for (row, link) in copy.links.iter().enumerate() {
        let matches: Vec<_> = members
            .iter()
            .filter(|member| same_physical_occurrence(&member.occurrence, &link.copied))
            .collect();
        inspect_tree_row(row, link, &matches, inspection);
    }
}
fn inspect_tree_row(
    row: usize,
    link: &crate::sketch_region_candidates::CandidateImageCopyLink,
    members: &[&crate::sketch_region_candidates::CandidateCopiedMember],
    inspection: &mut CandidateHistoryInspection,
) {
    if members.len() != 1 {
        retain_tree_fault(
            link,
            CandidateHistoryCause::CopiedTreeMultiplicity {
                row,
                count: members.len(),
            },
            inspection,
        );
        return;
    }
    inspect_tree_kind(row, link, members[0].kind, inspection);
}
fn inspect_tree_kind(
    row: usize,
    link: &crate::sketch_region_candidates::CandidateImageCopyLink,
    kind: rustycad_tkg3d::top_abs::ShapeEnum,
    inspection: &mut CandidateHistoryInspection,
) {
    match &link.kind {
        Ok(expected) if *expected != kind => {
            retain_tree_fault(link, CandidateHistoryCause::CopiedKind { row }, inspection)
        }
        Err(cause) => retain_tree_fault(
            link,
            CandidateHistoryCause::Access(cause.clone()),
            inspection,
        ),
        _ => {}
    }
}
fn retain_tree_fault(
    link: &crate::sketch_region_candidates::CandidateImageCopyLink,
    cause: CandidateHistoryCause,
    inspection: &mut CandidateHistoryInspection,
) {
    inspection.faults.push(CandidateHistoryFault {
        candidate: None,
        key: link.copied.clone(),
        cause,
    });
}
