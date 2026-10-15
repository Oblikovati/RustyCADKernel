// SPDX-License-Identifier: GPL-2.0-only
//! Shared completion of one isolated contact: source position and world construction.
use super::super::certificate_arithmetic::{RatioInterval, world::WorldEnvelope};
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::construction::WorldLimit;
use super::coordinate::{Coordinate, precision};
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::place::SourcePlace;
use super::report::{
    SketchContactTransition, SketchLineContact, SketchLinePosition, TargetParameter,
};
use super::source_frame::SourceFrame;
use super::spans::HomogeneousPoint;

/// Pair-specific certified facts before the source position is classified.
pub(super) struct ContactDraft {
    pub place: SourcePlace,
    pub target_parameter: Option<TargetParameter>,
    pub target_position: SketchLinePosition,
    pub transition: SketchContactTransition,
    pub multiplicity: [Option<usize>; 2],
    pub envelope: WorldEnvelope,
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
    source: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<SketchLineContact, Cause> {
    work.enter(SketchIncidenceStage::Classification);
    source.normalize(&mut draft.place, work)?;
    let source_position = source.position(&mut draft.place, work)?;
    Ok(SketchLineContact {
        target_index: owner.index,
        target_id: owner.id,
        place: draft.place,
        target_parameter: draft.target_parameter,
        source_position,
        target_position: draft.target_position,
        transition: draft.transition,
        multiplicity: draft.multiplicity,
        point: draft.envelope.point,
        world_bounds: draft.envelope.bounds,
        achieved_world_error: draft.envelope.achieved_error,
    })
}
/// Narrow the contact root until the point `(x, y) / w` meets the requested error.
pub(super) fn construct_point<T: Coordinate>(
    point: &HomogeneousPoint<T>,
    contact: &mut IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<WorldEnvelope, Cause> {
    work.enter(SketchIncidenceStage::Construction);
    let limit = WorldLimit::new(work.world_error());
    limit.construct(contact, work, &|root: &IsolatedRoot| {
        let [lower, upper] = root.ratio_bounds();
        point.enclose(&RatioInterval { lower, upper }, precision(root))
    })
}
