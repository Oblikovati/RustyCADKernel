// SPDX-License-Identifier: GPL-2.0-only
//! Original deletion is concluded empty only after all six directed uses reach kept cap outputs;
//! intermediate wire retirement is recorded separately and never renamed original deletion.
use super::super::complete_history::{observed_values, sweep_lineage};
use super::super::lineage::PrismLineageInspection;
use super::candidates::{AnnularCandidateHistory, AnnularCandidateWire};
use super::sources::AnnularSourceHistory;
use super::traces::AnnularGeneratedTrace;
use super::*;
use std::collections::BTreeSet;

/// One original directed use proved through every source, partition, candidate and sweep stage.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnnularOriginalContinuation {
    pub source: RegionSourceId,
    pub original: FrozenOriginalKey,
    pub original_use: PrismHistoryUseKey,
    pub input_copy: PrismHistoryUseKey,
    pub partition: PrismHistoryUseKey,
    pub candidate_copy: PrismHistoryUseKey,
    pub trace: AnnularGeneratedTrace,
    pub sweep_base: PrismHistoryUseKey,
    pub kept_first: Vec<FrozenUseKey>,
    pub kept_last: Vec<FrozenUseKey>,
}
/// One copied selected wire whose sweep query retired it; its original edge continues elsewhere.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnnularWireRetirement {
    pub source: RegionSourceId,
    pub selected_wire: u64,
    pub selected: PrismHistoryUseKey,
    pub sweep_base: PrismHistoryUseKey,
    pub parents: [Vec<FrozenOriginalKey>; 3],
}
pub(super) struct ContinuationInputs<'a> {
    pub scopes: &'a [AnnularBoundScope<'a>],
    pub sources: &'a [AnnularSourceHistory],
    pub candidate_copy: &'a [PrismHistoricalCopyQuery],
    pub candidates: &'a [AnnularCandidateHistory],
    pub traces: &'a [AnnularGeneratedTrace],
    pub lineage: &'a PrismLineageInspection,
    pub sweep: &'a [PrismCompleteSweepQuery],
}
/// Prove every original use of every authored source; the count must equal all original bindings.
pub(super) fn prove(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    inputs: &ContinuationInputs<'_>,
) -> HistoryCheck<Vec<AnnularOriginalContinuation>> {
    let mut result = Vec::new();
    for source in inputs.sources {
        for query in &source.partition_queries {
            let proved = continue_original(inputs, source, query);
            result.push(proved.map_err(|cause| cause.at_use(query.continued.clone()))?);
        }
    }
    let originals = request.publication.originals.len();
    let required = "all original directed uses proven to retained body";
    history_count(required, result.len(), originals)?;
    let distinct: BTreeSet<_> = result.iter().map(|r| &r.original).collect();
    let required = "distinct original use continuation census";
    history_count(required, distinct.len(), result.len())?;
    Ok(observed_values::sorted(result))
}
fn continue_original(
    inputs: &ContinuationInputs<'_>,
    source: &AnnularSourceHistory,
    query: &PrismPartitionQuery,
) -> HistoryCheck<AnnularOriginalContinuation> {
    let original_use = original_use(inputs, source.source, &query.original)?;
    let input_copy = copied_use(&source.initial_copy, &original_use)?;
    let candidate_copy = copied_use(inputs.candidate_copy, &query.continued)?;
    let trace = trace(inputs, &candidate_copy)?;
    admit_wire_source(inputs.candidates, trace.selected_wire, source.source)?;
    admit_wire_source(inputs.candidates, trace.rejected_wire, source.source)?;
    let sweep_base = sweep_lineage::copied_use(inputs.lineage, &trace.selected)?;
    let (kept_first, kept_last) = kept_outputs(inputs.sweep, &sweep_base, &query.original)?;
    Ok(AnnularOriginalContinuation {
        source: source.source,
        original: query.original.clone(),
        original_use,
        input_copy,
        partition: query.continued.clone(),
        candidate_copy,
        trace,
        sweep_base,
        kept_first,
        kept_last,
    })
}
fn original_use(
    inputs: &ContinuationInputs<'_>,
    source: RegionSourceId,
    original: &FrozenOriginalKey,
) -> HistoryCheck<PrismHistoryUseKey> {
    let scope = AnnularHistoryScope::Original(source);
    let scope = super::sources::bound_scope(inputs.scopes, scope)?;
    let named = |c: &&PrismHistoricalContext| {
        let last = c.key.0.last().map(|(_, id)| *id);
        c.occurrence.member.member == original.member && last == Some(original.directed_use)
    };
    let rows: Vec<_> = scope.owned.iter().filter(named).collect();
    let required = "one explicitly named original context in its own source scope";
    history_count(required, rows.len(), 1)?;
    holds(
        "original key belongs to its authored source",
        original.source == source.0,
    )?;
    Ok(rows[0].key.clone())
}
fn copied_use(
    rows: &[PrismHistoricalCopyQuery],
    source: &PrismHistoryUseKey,
) -> HistoryCheck<PrismHistoryUseKey> {
    let found: Vec<_> = rows.iter().filter(|q| &q.source == source).collect();
    let required = "one actual directed copy link in original continuation";
    history_count(required, found.len(), 1)?;
    Ok(found[0].copied.clone())
}
fn trace(
    inputs: &ContinuationInputs<'_>,
    copied: &PrismHistoryUseKey,
) -> HistoryCheck<AnnularGeneratedTrace> {
    let rows: Vec<_> = inputs
        .traces
        .iter()
        .filter(|r| &r.copied == copied)
        .collect();
    let required = "one generated continuation into selected and rejected domains";
    history_count(required, rows.len(), 1)?;
    Ok(rows[0].clone())
}
/// A continued boundary use lies on the authored wire of its own source role.
fn admit_wire_source(
    candidates: &[AnnularCandidateHistory],
    wire: u64,
    source: RegionSourceId,
) -> HistoryCheck<()> {
    let wires = candidates.iter().flat_map(|c| &c.wires);
    let rows: Vec<&AnnularCandidateWire> = wires.filter(|w| w.name == wire).collect();
    history_count("one authored wire per traced name", rows.len(), 1)?;
    let same = rows[0].source == source;
    holds("traced wire has the original's authored source role", same)
}
fn kept_outputs(
    queries: &[PrismCompleteSweepQuery],
    sweep_base: &PrismHistoryUseKey,
    original: &FrozenOriginalKey,
) -> HistoryCheck<(Vec<FrozenUseKey>, Vec<FrozenUseKey>)> {
    let rows: Vec<_> = queries.iter().filter(|r| &r.copied == sweep_base).collect();
    let required = "one exhaustive sweep query for original continuation";
    history_count(required, rows.len(), 1)?;
    let query = rows[0];
    let required = "original continuation is not copied-intermediate retirement";
    holds(required, !query.retired)?;
    let retained = query.parents.iter().flatten().any(|p| p == original);
    holds(
        "original continuation retains exact source parent",
        retained,
    )?;
    let first = cap(&query.first, "actual first cap continuation")?;
    let last = cap(&query.last, "actual last cap continuation")?;
    Ok((first, last))
}
fn cap(
    answer: &Option<PrismNativeSweepAnswer>,
    required: &'static str,
) -> HistoryCheck<Vec<FrozenUseKey>> {
    let answer = answer
        .as_ref()
        .ok_or(PrismHistoryCause::Missing { required })?;
    Ok(answer.outputs.clone())
}
/// Every retired sweep answer is a selected authored wire; each selected wire retires exactly once.
pub(super) fn retirements(
    inputs: &ContinuationInputs<'_>,
) -> HistoryCheck<Vec<AnnularWireRetirement>> {
    let selected_scope =
        |c: &&AnnularCandidateHistory| matches!(c.scope, AnnularHistoryScope::Selected(_));
    let candidates = inputs.candidates.iter().filter(selected_scope);
    let selected: Vec<_> = candidates.flat_map(|c| &c.wires).collect();
    let mut result = Vec::new();
    for query in inputs.sweep.iter().filter(|q| q.retired) {
        result.push(retirement(inputs, &selected, query)?);
    }
    let required = "one intermediate retirement per selected wire";
    history_count(required, result.len(), selected.len())?;
    let distinct: BTreeSet<_> = result.iter().map(|r| r.selected_wire).collect();
    history_count(
        "distinct retired selected wires",
        distinct.len(),
        selected.len(),
    )?;
    Ok(observed_values::sorted(result))
}
fn retirement(
    inputs: &ContinuationInputs<'_>,
    selected: &[&AnnularCandidateWire],
    query: &PrismCompleteSweepQuery,
) -> HistoryCheck<AnnularWireRetirement> {
    let directed = inputs.lineage.directed().iter();
    let same = |r: &&super::super::lineage::CopyDirectedObservation| {
        sweep_lineage::historical(&r.copied) == query.copied
    };
    let lineage: Vec<_> = directed.filter(same).collect();
    let required = "one selected lineage use for retired sweep query";
    history_count(required, lineage.len(), 1)?;
    let selected_key = sweep_lineage::historical(&lineage[0].selected);
    let wires: Vec<_> = selected.iter().filter(|w| w.wire == selected_key).collect();
    let required = "retired sweep query is one authored selected wire";
    history_count(required, wires.len(), 1)?;
    Ok(AnnularWireRetirement {
        source: wires[0].source,
        selected_wire: wires[0].name,
        selected: selected_key,
        sweep_base: query.copied.clone(),
        parents: query.parents.clone(),
    })
}
