// SPDX-License-Identifier: GPL-2.0-only
//! Physical lookup locates actual copy rows; the retained directed keys remain distinct observations.
// WHY: OCCT 3d097a0 BRepTools_Modifier.lxx:21 uses an IsSame-keyed map, and BRep_Tool::Pnt
// ignores orientation. This permits vertex image/point lookup, never directed occurrence equality.
use super::*;
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};
use crate::sketch_region_candidates::*;
use crate::sketch_region_vertex_observation::ObservedVertexPoint;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(crate) struct GeneratedContactObservation {
    pub ds_vertex: PartitionOccurrenceIdentity,
    /// Actual Modified query occurrence; it may differ in orientation from the DS vertex.
    pub native_image: PartitionOccurrenceIdentity,
    pub copied_image: PartitionOccurrenceIdentity,
    pub copied_index: usize,
    pub native_history: ContactImageAncestry,
    pub copied_point: ObservedVertexPoint,
    pub area: usize,
    pub directed_occurrence: PartitionOccurrenceIdentity,
    pub directed_point: ObservedVertexPoint,
    pub copied_history: ContactImageAncestry,
}

pub(crate) struct ContactImageAncestry {
    pub preserved: Vec<OriginalMemberIdentity>,
    pub modified: Vec<OriginalMemberIdentity>,
    pub generated: Vec<OriginalMemberIdentity>,
}
fn retain_image_ancestry(
    preserved: &[OriginalMemberIdentity],
    modified: &[OriginalMemberIdentity],
    generated: &[OriginalMemberIdentity],
) -> ContactImageAncestry {
    ContactImageAncestry {
        preserved: preserved.to_vec(),
        modified: modified.to_vec(),
        generated: generated.to_vec(),
    }
}

pub(super) fn retain_generated_vertices(retained: &mut PairedContactTransaction) {
    let Some(copied) = &retained.mapped.copied else {
        return;
    };
    for contact in &mut retained.inspection.contacts {
        for vertex in &mut contact.observed_vertices {
            vertex.generated_observations = observe_generated_vertex(copied, &vertex.occurrence);
        }
    }
}
fn observe_generated_vertex(
    copied: &CandidateCopyInspection,
    native: &PartitionOccurrenceIdentity,
) -> Vec<GeneratedContactObservation> {
    let members = match &copied.copied_tree {
        CandidateCopyTreeObservation::NotReached => return Vec::new(),
        CandidateCopyTreeObservation::Captured(rows)
        | CandidateCopyTreeObservation::Incomplete { members: rows, .. } => rows,
    };
    copied
        .links
        .iter()
        .filter(|link| {
            link.original.same_physical(native) && matches!(link.kind, Ok(ShapeEnum::Vertex))
        })
        .flat_map(|link| observe_copy_link(copied, members, link, native))
        .collect()
}
fn observe_copy_link(
    copied: &CandidateCopyInspection,
    members: &[CandidateCopiedMember],
    link: &CandidateImageCopyLink,
    native: &PartitionOccurrenceIdentity,
) -> Vec<GeneratedContactObservation> {
    let areas = match &copied.generated_topology {
        GeneratedTopologyObservation::NotReached => return Vec::new(),
        GeneratedTopologyObservation::Observed(rows)
        | GeneratedTopologyObservation::Incomplete { areas: rows, .. } => rows,
    };
    members
        .iter()
        .filter(|row| row.kind == ShapeEnum::Vertex && row.occurrence.same_physical(&link.copied))
        .flat_map(|member| observe_directed_members(areas, member, link, native))
        .collect()
}
fn observe_directed_members(
    areas: &[GeneratedAreaSnapshot],
    copied: &CandidateCopiedMember,
    link: &CandidateImageCopyLink,
    native: &PartitionOccurrenceIdentity,
) -> Vec<GeneratedContactObservation> {
    areas
        .iter()
        .flat_map(|area| {
            area.members
                .iter()
                .filter(|row| {
                    row.kind == ShapeEnum::Vertex
                        && row.copy_indices.contains(&copied.index)
                        && row.occurrence.same_physical(&link.copied)
                })
                .map(|directed| generated_contact_row(area.ordinal, copied, directed, link, native))
        })
        .collect()
}
fn generated_contact_row(
    area: usize,
    copied: &CandidateCopiedMember,
    directed: &GeneratedMemberSnapshot,
    link: &CandidateImageCopyLink,
    native: &PartitionOccurrenceIdentity,
) -> GeneratedContactObservation {
    GeneratedContactObservation {
        ds_vertex: native.clone(),
        native_image: link.original.clone(),
        native_history: native_image_ancestry(link),
        copied_image: link.copied.clone(),
        copied_index: copied.index,
        copied_point: copied.vertex_point.clone(),
        area,
        directed_occurrence: directed.occurrence.clone(),
        directed_point: directed.vertex_point.clone(),
        copied_history: directed_image_ancestry(directed),
    }
}

fn native_image_ancestry(link: &CandidateImageCopyLink) -> ContactImageAncestry {
    // WHY: native Generated must remain Generated through the subsequent copy.
    retain_image_ancestry(&link.preserved, &link.modified, &link.generated)
}
fn directed_image_ancestry(directed: &GeneratedMemberSnapshot) -> ContactImageAncestry {
    // WHY: directed output ancestry and the native map query describe different contexts.
    retain_image_ancestry(&directed.preserved, &directed.modified, &directed.generated)
}
