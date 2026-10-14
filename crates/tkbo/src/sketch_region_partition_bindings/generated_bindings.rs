// SPDX-License-Identifier: GPL-2.0-only
//! Stage history consistency is separate from accepted candidate or search coverage.
use super::{history_parents::*, history_types::*};
use crate::sketch_region_candidates::{
    CandidateCopiedMember, CandidateCopyInspection, CandidateCopyTreeObservation,
    CandidateImageCopyLink, GeneratedAreaSnapshot, GeneratedMemberSnapshot,
    GeneratedTopologyObservation, append_original_parents,
};

pub(crate) struct GeneratedBindingInspection {
    pub stage_observed: bool,
    pub faults: Vec<CandidateHistoryFault>,
}
pub(super) fn inspect_generated_bindings(
    copy: &CandidateCopyInspection,
) -> GeneratedBindingInspection {
    let mut inspection = GeneratedBindingInspection {
        stage_observed: false,
        faults: Vec::new(),
    };
    let areas = match &copy.generated_topology {
        GeneratedTopologyObservation::NotReached => return inspection,
        GeneratedTopologyObservation::Observed(areas) => {
            inspection.stage_observed = true;
            areas
        }
        GeneratedTopologyObservation::Incomplete { areas, .. } => areas,
    };
    for area in areas {
        inspect_generated_area(area, copy, &mut inspection);
    }
    inspection
}
fn inspect_generated_area(
    area: &GeneratedAreaSnapshot,
    copy: &CandidateCopyInspection,
    inspection: &mut GeneratedBindingInspection,
) {
    for (index, member) in area.members.iter().enumerate() {
        let mut faults = generated_member_faults(member, copy);
        append_duplicate_fault(area, index, &mut faults);
        for cause in faults {
            inspection.faults.push(CandidateHistoryFault {
                candidate: Some(area.ordinal),
                key: member.occurrence.clone(),
                cause,
            });
        }
    }
}
fn generated_member_faults(
    member: &GeneratedMemberSnapshot,
    copy: &CandidateCopyInspection,
) -> Vec<CandidateHistoryCause> {
    let expected = generated_expected_parents(member, &copy.links);
    let mut faults = Vec::new();
    for (kind, first, second) in [
        ("preserved", &expected.preserved, &member.preserved),
        ("modified", &expected.modified, &member.modified),
        ("generated", &expected.generated, &member.generated),
    ] {
        compare_parent_kind(kind, first, second, &mut faults);
    }
    inspect_generated_copy_indices(member, copy, &mut faults);
    faults
}
fn generated_expected_parents(
    member: &GeneratedMemberSnapshot,
    links: &[CandidateImageCopyLink],
) -> KindedBindingParents {
    let rows = physical_copy_rows(&member.occurrence, links);
    if !rows.is_empty() {
        return composed_parents(&rows, links);
    }
    generated_descendant_parents(member, links)
}
fn generated_descendant_parents(
    member: &GeneratedMemberSnapshot,
    links: &[CandidateImageCopyLink],
) -> KindedBindingParents {
    let rows = generated_descendant_rows(member, links);
    let direct = composed_parents(&rows, links);
    let mut expected = KindedBindingParents::default();
    append_original_parents(&mut expected.generated, &direct.modified);
    append_original_parents(&mut expected.generated, &direct.generated);
    expected
}
fn inspect_generated_copy_indices(
    member: &GeneratedMemberSnapshot,
    copy: &CandidateCopyInspection,
    faults: &mut Vec<CandidateHistoryCause>,
) {
    let CandidateCopyTreeObservation::Captured(tree) = &copy.copied_tree else {
        return;
    };
    let matches: Vec<_> = tree
        .iter()
        .filter(|row| row.occurrence.same_physical(&member.occurrence))
        .collect();
    if member.copy_indices != matches.iter().map(|row| row.index).collect::<Vec<_>>() {
        faults.push(CandidateHistoryCause::CopiedTreeMultiplicity {
            row: 0,
            count: matches.len(),
        });
    }
    inspect_generated_kind(member, &matches, faults);
}
fn inspect_generated_kind(
    member: &GeneratedMemberSnapshot,
    matches: &[&CandidateCopiedMember],
    faults: &mut Vec<CandidateHistoryCause>,
) {
    if matches.iter().any(|row| row.kind != member.kind) {
        faults.push(CandidateHistoryCause::CopiedKind { row: 0 });
    }
}

fn append_duplicate_fault(
    area: &GeneratedAreaSnapshot,
    index: usize,
    faults: &mut Vec<CandidateHistoryCause>,
) {
    let occurrence = &area.members[index].occurrence;
    let duplicates = area.members[..index]
        .iter()
        .filter(|row| row.occurrence == *occurrence)
        .count();
    if duplicates != 0 {
        faults.push(CandidateHistoryCause::RelationMultiplicity {
            count: duplicates + 1,
        });
    }
}

fn generated_descendant_rows(
    member: &GeneratedMemberSnapshot,
    links: &[CandidateImageCopyLink],
) -> Vec<usize> {
    links
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            member
                .descendants
                .iter()
                .any(|key| key.same_physical(&row.copied))
        })
        .map(|(index, _)| index)
        .collect()
}
