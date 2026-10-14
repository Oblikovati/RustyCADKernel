// SPDX-License-Identifier: GPL-2.0-only
//! Shared completion of one isolated contact: line position and world construction.
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::report::{SketchContactTransition, SketchLineContact, SketchLinePosition};
use super::source_line::SourceLine;

/// Target-specific certified facts before line classification and construction.
pub(super) struct ContactDraft {
    pub line_root: IsolatedRoot,
    pub target_root: Option<IsolatedRoot>,
    pub target_position: SketchLinePosition,
    pub transition: SketchContactTransition,
    pub multiplicity: [Option<usize>; 2],
}
/// Owner of the contact being completed.
#[derive(Clone, Copy)]
pub(super) struct TargetKey {
    pub index: usize,
    pub id: u64,
}
pub(super) fn finish(
    mut draft: ContactDraft,
    owner: TargetKey,
    source: &SourceLine,
    work: &mut Work<'_>,
) -> Result<SketchLineContact, Cause> {
    work.enter(SketchIncidenceStage::Classification);
    let line_position = source.position(&mut draft.line_root, work)?;
    work.enter(SketchIncidenceStage::Construction);
    let envelope = source.construct(&mut draft.line_root, work)?;
    Ok(SketchLineContact {
        target_index: owner.index,
        target_id: owner.id,
        line_root: draft.line_root,
        target_root: draft.target_root,
        line_position,
        target_position: draft.target_position,
        transition: draft.transition,
        multiplicity: draft.multiplicity,
        point: envelope.point,
        world_bounds: envelope.bounds,
        achieved_world_error: envelope.achieved_error,
    })
}
