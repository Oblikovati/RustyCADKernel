// SPDX-License-Identifier: GPL-2.0-only
//! Read the actual copier graph before retirement; physical indices are not canonical contacts.
use crate::sketch_region::{PartitionOccurrenceIdentity, equal_occurrences};
use rustycad_tkbrep::{topods::Shape, toptools::IndexedMapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

pub(crate) struct CandidateCopiedMember {
    pub index: usize,
    pub occurrence: PartitionOccurrenceIdentity,
    pub kind: ShapeEnum,
    pub vertex_point: crate::sketch_region_vertex_observation::ObservedVertexPoint,
}
pub(crate) enum CandidateCopyTreeObservation {
    NotReached,
    Captured(Vec<CandidateCopiedMember>),
    Incomplete {
        members: Vec<CandidateCopiedMember>,
        cause: Arc<Failure>,
    },
}
pub(super) fn snapshot_copied_tree(shape: &Shape) -> CandidateCopyTreeObservation {
    let mut members = Vec::new();
    match capture_copied_members(shape, &mut members) {
        Ok(()) => CandidateCopyTreeObservation::Captured(members),
        Err(cause) => CandidateCopyTreeObservation::Incomplete {
            members,
            cause: Arc::new(cause),
        },
    }
}
fn capture_copied_members(
    shape: &Shape,
    members: &mut Vec<CandidateCopiedMember>,
) -> Result<(), Failure> {
    let mut indexed = IndexedMapOfShape::default();
    // The mapped TopExp convenience traversal suppresses accessor errors; strict observation does not.
    for member in equal_occurrences(shape)? {
        indexed.add(member);
    }
    for index in 1..=indexed.extent() {
        let member = indexed.find_key(index)?;
        members.push(snapshot_copied_member(index, member)?);
    }
    Ok(())
}

fn snapshot_copied_member(index: usize, member: &Shape) -> Result<CandidateCopiedMember, Failure> {
    let occurrence = PartitionOccurrenceIdentity::capture(member);
    let kind = member.shape_type()?;
    Ok(CandidateCopiedMember {
        index,
        occurrence,
        kind,
        vertex_point: crate::sketch_region_vertex_observation::snapshot_vertex_point(member, kind),
    })
}
