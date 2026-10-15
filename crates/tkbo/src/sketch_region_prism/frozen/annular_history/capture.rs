// SPDX-License-Identifier: GPL-2.0-only
//! One capture composes the existing projections; it feeds both inspection and final reprojection.
use super::super::complete_history::{observed_values, sweep, sweep_lineage};
use super::super::lineage::PrismLineageInspection;
use super::candidates::{AnnularCandidateCopy, AnnularCandidateHistory};
use super::continuation::{AnnularOriginalContinuation, AnnularWireRetirement, ContinuationInputs};
use super::sources::{AnnularSourceHistory, bound_scope};
use super::sweep_parents::AnnularCapturedSweepParents;
use super::traces::AnnularGeneratedTrace;
use super::*;

/// The authored member keys of one semantic scope.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnnularScopeRoster {
    pub scope: AnnularHistoryScope,
    pub members: Vec<PrismHistoryMemberKey>,
}
pub(super) struct CapturedAnnularHistory {
    pub contexts: Vec<PrismHistoricalContext>,
    pub rosters: Vec<AnnularScopeRoster>,
    pub sources: Vec<AnnularSourceHistory>,
    pub partition_relations: Vec<PrismHistoricalRelation>,
    pub candidate_copy: AnnularCandidateCopy,
    pub candidates: Vec<AnnularCandidateHistory>,
    pub traces: Vec<AnnularGeneratedTrace>,
    pub lineage: PrismLineageInspection,
    pub sweep: Vec<PrismCompleteSweepQuery>,
    pub captured_parents: Vec<AnnularCapturedSweepParents>,
    pub continued: Vec<AnnularOriginalContinuation>,
    pub retired: Vec<AnnularWireRetirement>,
}
struct Intermediate {
    sources: Vec<AnnularSourceHistory>,
    partition_relations: Vec<PrismHistoricalRelation>,
    candidate_copy: AnnularCandidateCopy,
    candidates: Vec<AnnularCandidateHistory>,
    traces: Vec<AnnularGeneratedTrace>,
}
struct Swept {
    lineage: PrismLineageInspection,
    sweep: Vec<PrismCompleteSweepQuery>,
    captured_parents: Vec<AnnularCapturedSweepParents>,
}
/// Capture every scope, intermediate disposition, sweep answer and original continuation.
pub(super) fn capture(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
) -> HistoryCheck<CapturedAnnularHistory> {
    let operands = super::scopes::capture_operands(request)?;
    let scopes = super::names::bind_scopes(request, &operands)?;
    let intermediate = intermediate(request, &scopes)?;
    let swept = swept(request, &scopes)?;
    let inputs = ContinuationInputs {
        scopes: &scopes,
        sources: &intermediate.sources,
        candidate_copy: &intermediate.candidate_copy.queries,
        candidates: &intermediate.candidates,
        traces: &intermediate.traces,
        lineage: &swept.lineage,
        sweep: &swept.sweep,
    };
    let continued = super::continuation::prove(request, &inputs)?;
    let retired = super::continuation::retirements(&inputs)?;
    Ok(finish(&scopes, intermediate, swept, continued, retired))
}
fn intermediate(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
) -> HistoryCheck<Intermediate> {
    let sources = super::sources::capture(request, scopes)?;
    let partition = bound_scope(scopes, AnnularHistoryScope::Partition)?;
    let partition_relations = super::partition::capture(request, partition, &sources)?;
    let candidate_copy = super::candidates::copy(request, scopes, &partition_relations)?;
    let candidates = super::candidates::capture(request, scopes, &candidate_copy)?;
    let traces = super::traces::capture(scopes, &candidates)?;
    Ok(Intermediate {
        sources,
        partition_relations,
        candidate_copy,
        candidates,
        traces,
    })
}
fn swept(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
) -> HistoryCheck<Swept> {
    let bound = super::super::bindings::bind_projection(request.receipt, request.publication)
        .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))?;
    let lineage =
        sweep_lineage::capture_projection(&bound, request.joined.projection(), request.copied)?;
    let base = bound_scope(scopes, AnnularHistoryScope::SweepBase)?;
    let sweep = sweep::capture_native(&request.native_inputs(), &bound, base)?;
    let captured_parents = super::sweep_parents::capture(request, &bound, base, &sweep)?;
    Ok(Swept {
        lineage,
        sweep,
        captured_parents,
    })
}
fn finish(
    scopes: &[AnnularBoundScope<'_>],
    intermediate: Intermediate,
    swept: Swept,
    continued: Vec<AnnularOriginalContinuation>,
    retired: Vec<AnnularWireRetirement>,
) -> CapturedAnnularHistory {
    CapturedAnnularHistory {
        contexts: observed_values::sorted(
            scopes
                .iter()
                .flat_map(|s| s.contexts.owned.clone())
                .collect(),
        ),
        rosters: observed_values::sorted(scopes.iter().map(roster).collect()),
        sources: intermediate.sources,
        partition_relations: intermediate.partition_relations,
        candidate_copy: intermediate.candidate_copy,
        candidates: intermediate.candidates,
        traces: intermediate.traces,
        lineage: swept.lineage,
        sweep: swept.sweep,
        captured_parents: swept.captured_parents,
        continued,
        retired,
    }
}
fn roster(scope: &AnnularBoundScope<'_>) -> AnnularScopeRoster {
    let mut members: Vec<_> = scope
        .contexts
        .owned
        .iter()
        .map(|c| c.occurrence.member)
        .collect();
    members.sort();
    members.dedup();
    AnnularScopeRoster {
        scope: scope.scope,
        members,
    }
}
