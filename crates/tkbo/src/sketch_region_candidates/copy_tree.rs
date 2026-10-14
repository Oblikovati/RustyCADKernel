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
        members.push(CandidateCopiedMember {
            index,
            occurrence: PartitionOccurrenceIdentity::capture(member),
            kind: member.shape_type()?,
        });
    }
    Ok(())
}
