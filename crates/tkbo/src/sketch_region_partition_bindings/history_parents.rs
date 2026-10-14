// SPDX-License-Identifier: GPL-2.0-only
//! Compose only actual matched copy rows; parent overlap cannot recover output identity.
use super::{history_types::*, occurrence_keys::same_physical_occurrence};
use crate::sketch_region::{
    ArrangementRelation, OriginalMemberIdentity, PartitionOccurrenceIdentity, equal_occurrences,
};
use crate::sketch_region_candidates::{CandidateImageCopyLink, append_original_parents};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

pub(super) fn physical_copy_rows(
    key: &PartitionOccurrenceIdentity,
    links: &[CandidateImageCopyLink],
) -> Vec<usize> {
    links
        .iter()
        .enumerate()
        .filter(|(_, row)| same_physical_occurrence(key, &row.copied))
        .map(|(index, _)| index)
        .collect()
}
pub(super) fn expected_history(
    member: &Shape,
    links: &[CandidateImageCopyLink],
) -> Result<(Vec<usize>, KindedBindingParents), Failure> {
    let direct = physical_copy_rows(&PartitionOccurrenceIdentity::capture(member), links);
    if !direct.is_empty() {
        return Ok((direct.clone(), composed_parents(&direct, links)));
    }
    let kind = member.shape_type()?;
    if !matches!(
        kind,
        rustycad_tkg3d::top_abs::ShapeEnum::Face | rustycad_tkg3d::top_abs::ShapeEnum::Wire
    ) {
        return Ok((direct, KindedBindingParents::default()));
    }
    descendant_history(member, links)
}
fn descendant_history(
    member: &Shape,
    links: &[CandidateImageCopyLink],
) -> Result<(Vec<usize>, KindedBindingParents), Failure> {
    let mut rows = Vec::new();
    for child in equal_occurrences(member)?.iter().skip(1) {
        append_copy_rows(
            &mut rows,
            physical_copy_rows(&PartitionOccurrenceIdentity::capture(child), links),
        );
    }
    let direct = composed_parents(&rows, links);
    let mut parents = KindedBindingParents::default();
    append_original_parents(&mut parents.generated, &direct.modified);
    append_original_parents(&mut parents.generated, &direct.generated);
    Ok((rows, parents))
}
pub(super) fn composed_parents(
    rows: &[usize],
    links: &[CandidateImageCopyLink],
) -> KindedBindingParents {
    let mut parents = KindedBindingParents::default();
    for &index in rows {
        let row = &links[index];
        append_original_parents(&mut parents.modified, &row.preserved);
        append_original_parents(&mut parents.modified, &row.modified);
        append_original_parents(&mut parents.generated, &row.generated);
    }
    parents
}
pub(super) fn history_parent_faults(
    expected: &KindedBindingParents,
    actual: &ArrangementRelation,
) -> Vec<CandidateHistoryCause> {
    let mut faults = Vec::new();
    for (kind, first, second) in [
        ("preserved", &expected.preserved, &actual.preserved),
        ("modified", &expected.modified, &actual.modified),
        ("generated", &expected.generated, &actual.generated),
    ] {
        compare_parent_kind(kind, first, second, &mut faults);
    }
    faults
}
pub(super) fn compare_parent_kind(
    kind: &'static str,
    expected: &[OriginalMemberIdentity],
    actual: &[OriginalMemberIdentity],
    faults: &mut Vec<CandidateHistoryCause>,
) {
    if actual
        .iter()
        .enumerate()
        .any(|(index, parent)| actual[..index].contains(parent))
    {
        faults.push(CandidateHistoryCause::DuplicateParents { kind });
    }
    if expected.len() != actual.len() || expected.iter().any(|parent| !actual.contains(parent)) {
        faults.push(CandidateHistoryCause::KindedParents { kind });
    }
}

fn append_copy_rows(rows: &mut Vec<usize>, incoming: Vec<usize>) {
    for index in incoming {
        if !rows.contains(&index) {
            rows.push(index);
        }
    }
}
