// SPDX-License-Identifier: GPL-2.0-only
//! Owned scalar capture composes existing proof observations without another geometry evaluator.
use super::*;
pub(super) struct CapturedPrismHistory {
    pub intermediate: PrismIntermediateHistory,
    pub partition: PrismPartitionMetadata,
    pub lineage: super::super::lineage::PrismLineageInspection,
    pub sweep: Vec<PrismCompleteSweepQuery>,
    pub continued: Vec<PrismOriginalContinuation>,
}
pub(super) fn capture(request: &PrismHistoryRequest<'_, '_>) -> HistoryCheck<CapturedPrismHistory> {
    let operands = stages::capture_operands(request)?;
    let stages = stages::bind_stages(request, &operands)?;
    let intermediate = intermediate::capture(request, &stages)?;
    let partition = partition_metadata::capture(
        request,
        stages::stage(&stages, PrismHistoryStage::InputCopy)?,
    )?;
    let (lineage, sweep, continued) = capture_kept_history(request, &stages, &intermediate)?;
    Ok(CapturedPrismHistory {
        intermediate,
        partition,
        lineage,
        sweep,
        continued,
    })
}

fn capture_kept_history(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[stages::BoundHistoryStage<'_>],
    intermediate: &PrismIntermediateHistory,
) -> HistoryCheck<(
    super::super::lineage::PrismLineageInspection,
    Vec<PrismCompleteSweepQuery>,
    Vec<PrismOriginalContinuation>,
)> {
    let bound = super::super::bindings::bind_projection(request.receipt, request.publication)
        .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))?;
    let lineage = sweep_lineage::capture(request, &bound)?;
    let sweep = sweep::capture(
        request,
        &bound,
        stages::stage(stages, PrismHistoryStage::SweepBase)?,
    )?;
    let continued = continuation::prove(request, intermediate, &lineage, &sweep)?;
    Ok((lineage, sweep, continued))
}
