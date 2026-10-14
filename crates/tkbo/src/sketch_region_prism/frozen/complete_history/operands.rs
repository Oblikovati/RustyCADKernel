// SPDX-License-Identifier: GPL-2.0-only
//! Capture live roots once; historical query snapshots remain separate from current geometry.
use super::*;
use crate::sketch_region::{
    OwnedPartitionSnapshot, PartitionObservation, RegionContextObservation,
    RegionContextReadFailure, RegionCopyContextObservation, RegionCopyContexts,
    RegionObservedContext,
};
use crate::sketch_region_candidates::{CandidateBatch, RegionCandidate};
use rustycad_tkbrep::topods::Shape;

pub(super) struct HistoryStageOperands {
    pub stage: PrismHistoryStage,
    pub rows: Vec<RegionObservedContext>,
}
pub(super) fn current_contexts(
    root: &Shape,
    stage: PrismHistoryStage,
    limits: PrismHistoryLimits,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    bounds::preflight_history_root(root, limits).map_err(PrismHistoryCause::Bounds)?;
    observed_contexts(&crate::sketch_region::observe_region_contexts(root), stage).cloned()
}
pub(super) fn observed_contexts(
    rows: &RegionContextObservation,
    stage: PrismHistoryStage,
) -> HistoryCheck<&Vec<RegionObservedContext>> {
    match rows {
        RegionContextObservation::Captured(rows) => Ok(rows),
        RegionContextObservation::NotReached => Err(PrismHistoryCause::ContextRead {
            stage,
            cause: HistoricalContextReadCause::NotReached,
        }),
        RegionContextObservation::Incomplete(cause) => Err(context_failure(stage, cause)),
    }
}
pub(super) fn copied_contexts(
    rows: &RegionCopyContextObservation,
    stage: PrismHistoryStage,
) -> HistoryCheck<&RegionCopyContexts> {
    match rows {
        RegionCopyContextObservation::Captured(rows) => Ok(rows),
        RegionCopyContextObservation::NotReached => Err(PrismHistoryCause::ContextRead {
            stage,
            cause: HistoricalContextReadCause::NotReached,
        }),
        RegionCopyContextObservation::Incomplete(cause) => Err(context_failure(stage, cause)),
    }
}
fn context_failure(
    stage: PrismHistoryStage,
    cause: &RegionContextReadFailure,
) -> PrismHistoryCause {
    match cause {
        RegionContextReadFailure::Native(cause) => PrismHistoryCause::Native(cause.clone()),
        RegionContextReadFailure::RepeatedAncestor(_) => PrismHistoryCause::ContextRead {
            stage,
            cause: HistoricalContextReadCause::RepeatedAncestor,
        },
    }
}
pub(super) fn partition<'a>(
    request: &PrismHistoryRequest<'a>,
) -> HistoryCheck<&'a OwnedPartitionSnapshot> {
    let PartitionObservation::Captured(rows) = &request.bridge.mapped.arrangement.partition else {
        return Err(PrismHistoryCause::Missing {
            required: "complete observed partition",
        });
    };
    Ok(rows)
}
pub(super) fn candidates<'a>(
    request: &PrismHistoryRequest<'a>,
) -> HistoryCheck<&'a CandidateBatch> {
    request
        .bridge
        .mapped
        .copied
        .as_ref()
        .and_then(|copy| copy.completion.as_ref().ok())
        .ok_or(PrismHistoryCause::Missing {
            required: "actual completed candidate batch",
        })
}
pub(super) fn candidate<'a>(
    request: &PrismHistoryRequest<'a>,
    stage: PrismHistoryStage,
) -> HistoryCheck<&'a RegionCandidate> {
    let region = &request.joined.embedding().binding().region;
    let face = if stage == PrismHistoryStage::Selected {
        &region.source_candidate.face
    } else {
        &region.complementary_candidate.face
    };
    let found: Vec<_> = candidates(request)?
        .candidates
        .iter()
        .filter(|c| crate::sketch_region::PartitionOccurrenceIdentity::capture(&c.face) == *face)
        .collect();
    history_count(
        "one current candidate for certified semantic region",
        found.len(),
        1,
    )?;
    Ok(found[0])
}
pub(super) fn generated_contexts(
    request: &PrismHistoryRequest<'_>,
    stage: PrismHistoryStage,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    let actual = current_contexts(
        &candidate(request, stage)?.face,
        stage,
        request.requested.limits,
    )?;
    let mut matched = 0;
    for area in generated::areas(request)? {
        if observed_contexts(&area.contexts, stage)? == &actual {
            matched += 1;
        }
    }
    history_count(
        "one generated census equals current semantic candidate",
        matched,
        1,
    )?;
    Ok(actual)
}
