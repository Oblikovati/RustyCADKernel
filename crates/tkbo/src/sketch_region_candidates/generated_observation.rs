// SPDX-License-Identifier: GPL-2.0-only
//! Builder-stage observations preserve INTERNAL members without admitting candidate coverage.
use super::{copy_tree::*, generation::CandidateParents, outcome::*};
use crate::bop_algo::BuilderFace;
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(crate) struct GeneratedMemberSnapshot {
    pub occurrence: PartitionOccurrenceIdentity,
    pub kind: ShapeEnum,
    pub descendants: Vec<PartitionOccurrenceIdentity>,
    pub copy_indices: Vec<usize>,
    pub preserved: Vec<OriginalMemberIdentity>,
    pub modified: Vec<OriginalMemberIdentity>,
    pub generated: Vec<OriginalMemberIdentity>,
}
pub(crate) struct GeneratedAreaSnapshot {
    pub ordinal: usize,
    pub members: Vec<GeneratedMemberSnapshot>,
}
pub(crate) enum GeneratedTopologyObservation {
    NotReached,
    Observed(Vec<GeneratedAreaSnapshot>),
    Incomplete {
        areas: Vec<GeneratedAreaSnapshot>,
        cause_index: usize,
    },
}
pub(super) fn snapshot_generated_areas(
    builder: &BuilderFace,
    parents: &CandidateParents,
    tree: &CandidateCopyTreeObservation,
    failures: &mut Vec<CandidateCause>,
) -> GeneratedTopologyObservation {
    let mut areas = Vec::new();
    match capture_generated_areas(builder, parents, tree, &mut areas) {
        Ok(()) => GeneratedTopologyObservation::Observed(areas),
        Err(cause) => retain_generated_failure(areas, cause, failures),
    }
}
fn retain_generated_failure(
    areas: Vec<GeneratedAreaSnapshot>,
    cause: CandidateCause,
    failures: &mut Vec<CandidateCause>,
) -> GeneratedTopologyObservation {
    let cause_index = failures.len();
    failures.push(cause);
    GeneratedTopologyObservation::Incomplete { areas, cause_index }
}
fn capture_generated_areas(
    builder: &BuilderFace,
    parents: &CandidateParents,
    tree: &CandidateCopyTreeObservation,
    areas: &mut Vec<GeneratedAreaSnapshot>,
) -> Result<(), CandidateCause> {
    for (ordinal, area) in builder.areas().iter().enumerate() {
        areas.push(GeneratedAreaSnapshot {
            ordinal,
            members: Vec::new(),
        });
        let snapshot = areas.last_mut().expect("generated area was appended");
        capture_generated_members(area, parents, tree, &mut snapshot.members)?;
    }
    Ok(())
}
fn capture_generated_members(
    area: &rustycad_tkbrep::topods::Shape,
    parents: &CandidateParents,
    tree: &CandidateCopyTreeObservation,
    members: &mut Vec<GeneratedMemberSnapshot>,
) -> Result<(), CandidateCause> {
    for relation in super::ancestry::candidate_relations(area, parents)? {
        members.push(snapshot_generated_member(relation, tree)?);
    }
    Ok(())
}
fn actual_copy_indices(
    tree: &CandidateCopyTreeObservation,
    occurrence: &PartitionOccurrenceIdentity,
) -> Vec<usize> {
    let members = match tree {
        CandidateCopyTreeObservation::NotReached => return Vec::new(),
        CandidateCopyTreeObservation::Captured(members)
        | CandidateCopyTreeObservation::Incomplete { members, .. } => members,
    };
    members
        .iter()
        .filter(|row| row.occurrence.same_physical(occurrence))
        .map(|row| row.index)
        .collect()
}

fn snapshot_generated_member(
    relation: crate::sketch_region::ArrangementRelation,
    tree: &CandidateCopyTreeObservation,
) -> Result<GeneratedMemberSnapshot, CandidateCause> {
    let occurrence = PartitionOccurrenceIdentity::capture(&relation.output);
    Ok(GeneratedMemberSnapshot {
        copy_indices: actual_copy_indices(tree, &occurrence),
        occurrence,
        kind: relation
            .output
            .shape_type()
            .map_err(CandidateCause::Mapped)?,
        descendants: generated_descendants(&relation.output)?,
        preserved: relation.preserved,
        modified: relation.modified,
        generated: relation.generated,
    })
}

fn generated_descendants(
    shape: &rustycad_tkbrep::topods::Shape,
) -> Result<Vec<PartitionOccurrenceIdentity>, CandidateCause> {
    Ok(crate::sketch_region::equal_occurrences(shape)
        .map_err(CandidateCause::Mapped)?
        .iter()
        .skip(1)
        .map(PartitionOccurrenceIdentity::capture)
        .collect())
}
