// SPDX-License-Identifier: GPL-2.0-only
//! Existing retained observations are located by actual source identity, never returned rank.
use super::*;
use crate::sketch_region::{ArrangementCompletion, OwnedPartitionSnapshot, PartitionObservation};
use rustycad_tkbrep::topods::Shape;
pub(super) fn partition(
    prepared: &PreparedPolynomialPrism,
) -> ResolveCheck<&OwnedPartitionSnapshot> {
    let bridge = require_local(
        prepared.retained.bridge.as_ref(),
        "retained contact transaction",
    )?;
    let PartitionObservation::Captured(partition) = &bridge.mapped.arrangement.partition else {
        return Err(ResolutionCause::Missing {
            required: "complete actual partition observation",
        });
    };
    Ok(partition)
}
pub(super) fn initial_copy(
    prepared: &PreparedPolynomialPrism,
) -> ResolveCheck<&RegionCopyContexts> {
    let partition = partition(prepared)?;
    let identity = initial_source_identity(prepared, partition)?;
    let found: Vec<_> = partition
        .input_contexts
        .iter()
        .filter(|row| row.source == identity)
        .collect();
    require_count("initial copy belongs to constructed source", found.len(), 1)?;
    Ok(retained_copy_contexts(
        &found[0].observation,
        PrismHistoryStage::InputCopy,
    )?)
}
pub(super) fn candidate_copy(
    prepared: &PreparedPolynomialPrism,
) -> ResolveCheck<&RegionCopyContexts> {
    let bridge = require_local(
        prepared.retained.bridge.as_ref(),
        "retained contact transaction",
    )?;
    let copied = require_local(bridge.mapped.copied.as_ref(), "retained candidate copy")?;
    Ok(retained_copy_contexts(
        &copied.contexts,
        PrismHistoryStage::CandidateCopy,
    )?)
}
pub(super) fn partition_root(prepared: &PreparedPolynomialPrism) -> ResolveCheck<&Shape> {
    let bridge = require_local(
        prepared.retained.bridge.as_ref(),
        "retained contact transaction",
    )?;
    let ArrangementCompletion::Arranged(arranged) = &bridge.mapped.arrangement.completion else {
        return Err(ResolutionCause::Missing {
            required: "completed arranged partition root",
        });
    };
    Ok(&arranged.arrangement)
}
pub(super) fn current_partition(
    prepared: &PreparedPolynomialPrism,
) -> ResolveCheck<Vec<RegionObservedContext>> {
    let root = partition_root(prepared)?;
    let limits = prepared.retained.request.policy.publication.history.limits;
    let actual = capture_history_contexts(root, PrismHistoryStage::Partition, limits)?;
    require_count(
        "current partition equals actual candidate-copy source census",
        usize::from(actual == candidate_copy(prepared)?.source),
        1,
    )?;
    admit_history_stage_contexts(PrismHistoryStage::Partition, &actual, limits)?;
    Ok(actual)
}

fn initial_source_identity(
    prepared: &PreparedPolynomialPrism,
    partition: &OwnedPartitionSnapshot,
) -> ResolveCheck<crate::sketch_region::RegionSourceId> {
    let source = require_local(
        prepared.retained.source.as_ref(),
        "constructed source identity",
    )?;
    require_count(
        "one actual initial source copy",
        partition.input_contexts.len(),
        1,
    )?;
    Ok(source.identity)
}
