// SPDX-License-Identifier: GPL-2.0-only
//! One explicit semantic roster; no returned ordinal supplies a stage or a member name.
use super::*;
use crate::sketch_region::{ArrangementCompletion, RegionObservedContext};
use operands::HistoryStageOperands;

pub(super) const STAGES: [PrismHistoryStage; 7] = [
    PrismHistoryStage::Original,
    PrismHistoryStage::InputCopy,
    PrismHistoryStage::Partition,
    PrismHistoryStage::CandidateCopy,
    PrismHistoryStage::Selected,
    PrismHistoryStage::Complement,
    PrismHistoryStage::SweepBase,
];
pub(super) struct BoundHistoryStage<'a> {
    pub stage: PrismHistoryStage,
    pub contexts: BoundHistoricalContexts<'a>,
}
pub(super) fn capture_operands(
    request: &PrismHistoryRequest<'_, '_>,
) -> HistoryCheck<Vec<HistoryStageOperands>> {
    let initial = initial_contexts(request)?;
    let mut result = Vec::new();
    for stage in STAGES {
        let rows = rows_for(request, stage, initial)?;
        census::admit_stage(stage, &rows, request.requested.limits)?;
        result.push(HistoryStageOperands { stage, rows });
    }
    Ok(result)
}
fn rows_for(
    request: &PrismHistoryRequest<'_, '_>,
    stage: PrismHistoryStage,
    initial: &crate::sketch_region::RegionCopyContexts,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    match stage {
        PrismHistoryStage::Original => original_rows(request, initial),
        PrismHistoryStage::InputCopy => Ok(initial.copied.clone()),
        PrismHistoryStage::Partition => partition_rows(request),
        PrismHistoryStage::CandidateCopy => Ok(candidate_copy(request)?.copied.clone()),
        PrismHistoryStage::Selected | PrismHistoryStage::Complement => {
            operands::generated_contexts(request, stage)
        }
        PrismHistoryStage::SweepBase => sweep_rows(request),
    }
}
fn original_rows(
    request: &PrismHistoryRequest<'_, '_>,
    initial: &crate::sketch_region::RegionCopyContexts,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    let rows = operands::current_contexts(
        request.sources[0].shape,
        PrismHistoryStage::Original,
        request.requested.limits,
    )?;
    history_count(
        "current source exactly matches historical original context census",
        usize::from(rows == initial.source),
        1,
    )?;
    Ok(rows)
}
fn partition_rows(
    request: &PrismHistoryRequest<'_, '_>,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    let ArrangementCompletion::Arranged(owned) = &request.bridge.mapped.arrangement.completion
    else {
        return Err(PrismHistoryCause::Missing {
            required: "actual arranged partition root",
        });
    };
    let rows = operands::current_contexts(
        &owned.arrangement,
        PrismHistoryStage::Partition,
        request.requested.limits,
    )?;
    history_count(
        "current partition equals copier source context census",
        usize::from(rows == candidate_copy(request)?.source),
        1,
    )?;
    Ok(rows)
}
fn sweep_rows(request: &PrismHistoryRequest<'_, '_>) -> HistoryCheck<Vec<RegionObservedContext>> {
    let copied = request
        .receipt
        .copied
        .as_ref()
        .ok_or(PrismHistoryCause::Missing {
            required: "actual sweep copied base",
        })?;
    operands::current_contexts(
        &copied.base,
        PrismHistoryStage::SweepBase,
        request.requested.limits,
    )
}
pub(super) fn candidate_copy<'a>(
    request: &PrismHistoryRequest<'a, '_>,
) -> HistoryCheck<&'a crate::sketch_region::RegionCopyContexts> {
    let copied = request
        .bridge
        .mapped
        .copied
        .as_ref()
        .ok_or(PrismHistoryCause::Missing {
            required: "actual candidate copy receipt",
        })?;
    operands::copied_contexts(&copied.contexts, PrismHistoryStage::CandidateCopy)
}
pub(super) fn bind_stages<'a>(
    request: &PrismHistoryRequest<'_, '_>,
    operands: &'a [HistoryStageOperands],
) -> HistoryCheck<Vec<BoundHistoryStage<'a>>> {
    history_count(
        "exact four additional authored stage bindings",
        request.historical.intermediate.len(),
        4,
    )?;
    let mut result = Vec::new();
    for operand in operands {
        let contexts = bind_stage(request, operand)?;
        result.push(BoundHistoryStage {
            stage: operand.stage,
            contexts,
        });
    }
    Ok(result)
}
fn bind_stage<'a>(
    request: &PrismHistoryRequest<'_, '_>,
    operand: &'a HistoryStageOperands,
) -> HistoryCheck<BoundHistoricalContexts<'a>> {
    let names = match operand.stage {
        PrismHistoryStage::Original => stage_names::original_names(request, &operand.rows)?,
        PrismHistoryStage::Selected | PrismHistoryStage::SweepBase => {
            stage_names::legacy_names(request, operand.stage, &operand.rows)?
        }
        _ => {
            return contexts::bind_historical_contexts(
                request.publication.operation,
                authored_stage(request, operand.stage)?,
                &operand.rows,
                request.requested.limits,
            );
        }
    };
    contexts::bind_historical_contexts(
        request.publication.operation,
        &names,
        &operand.rows,
        request.requested.limits,
    )
}
fn authored_stage<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    stage: PrismHistoryStage,
) -> HistoryCheck<&'a PrismHistoricalStageBindings> {
    let rows: Vec<_> = request
        .historical
        .intermediate
        .iter()
        .filter(|b| b.stage == stage)
        .collect();
    history_count(
        "one authored binding for required intermediate stage",
        rows.len(),
        1,
    )?;
    Ok(rows[0])
}
pub(super) fn stage<'a, 'b>(
    rows: &'a [BoundHistoryStage<'b>],
    stage: PrismHistoryStage,
) -> HistoryCheck<&'a BoundHistoricalContexts<'b>> {
    let matches: Vec<_> = rows.iter().filter(|row| row.stage == stage).collect();
    history_count("one bound semantic history stage", matches.len(), 1)?;
    Ok(&matches[0].contexts)
}

fn initial_contexts<'a>(
    request: &PrismHistoryRequest<'a, '_>,
) -> HistoryCheck<&'a crate::sketch_region::RegionCopyContexts> {
    history_count("one original Jordan source", request.sources.len(), 1)?;
    let partition = operands::partition(request)?;
    history_count(
        "one initial copied source context receipt",
        partition.input_contexts.len(),
        1,
    )?;
    history_count(
        "same caller-authored input source",
        usize::from(partition.input_contexts[0].source == request.sources[0].identity),
        1,
    )?;
    let initial = operands::copied_contexts(
        &partition.input_contexts[0].observation,
        PrismHistoryStage::InputCopy,
    )?;
    Ok(initial)
}
