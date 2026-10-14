// SPDX-License-Identifier: GPL-2.0-only
//! Stable authored keys order records; no lookup address, graph index or returned ordinal is encoded.
use super::super::complete_history::partition_metadata;
use super::capture::CapturedAnnularHistory;
use super::result::*;
use super::sources::AnnularSourceHistory;
use super::*;

/// Seal one captured history with its measured units and the complete joined bytes.
pub(super) fn finish(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    captured: CapturedAnnularHistory,
    measured_operand_rows: usize,
) -> AnnularCompleteHistory {
    let mut result = AnnularCompleteHistory {
        binding: source_binding(request, &captured),
        requested: request.requested,
        measured_operand_rows,
        captured,
        joined: request.joined.replay_bytes().to_vec(),
        replay: Vec::new(),
        deleted: Vec::new(),
    };
    result.replay = encode(&result);
    result
}
fn source_binding(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    captured: &CapturedAnnularHistory,
) -> AnnularHistorySourceBinding {
    let sources = super::scopes::source_roles(request).into_iter();
    let source = |source| AnnularHistorySource {
        source,
        originals: originals(captured, source),
    };
    let rejected = captured.candidates.iter();
    let rejected = rejected.filter(|c| matches!(c.scope, AnnularHistoryScope::Rejected(_)));
    AnnularHistorySourceBinding {
        operation: request.publication.operation,
        selected_region: request.publication.selected_region,
        selected_candidate: request.region.identities.selected,
        rejected_candidates: rejected.map(|c| c.name).collect(),
        sources: sources.map(source).collect(),
    }
}
fn originals(captured: &CapturedAnnularHistory, source: RegionSourceId) -> Vec<FrozenOriginalKey> {
    let rows = captured.continued.iter().filter(|c| c.source == source);
    rows.map(|c| c.original.clone()).collect()
}
fn block(bytes: &[u8], result: &mut Vec<u8>) {
    result.extend((bytes.len() as u64).to_le_bytes());
    result.extend(bytes);
}
fn encode(value: &AnnularCompleteHistory) -> Vec<u8> {
    let c = &value.captured;
    let mut result = format!(
        "martensite-complete-annular-history-v2;binding={:?};requested={:?};effective={:?};measured-operand-rows={:?};rosters={:?};contexts={:?};partition-relations={:?};candidate-copy={:?};candidates={:?};traces={:?};sweep={:?};captured-parents={:?};continued={:?};retired-wires={:?};original-deleted={:?};member-coverage=complete;directed-coverage=complete;whole-body-validation=not-claimed;",
        value.binding, value.requested, value.requested, value.measured_operand_rows, c.rosters, c.contexts,
        c.partition_relations, c.candidate_copy, c.candidates, c.traces, c.sweep, c.captured_parents,
        c.continued, c.retired, value.deleted
    )
    .into_bytes();
    for source in &c.sources {
        block(&source_bytes(source), &mut result);
    }
    block(&c.lineage.replay_bytes(), &mut result);
    block(&value.joined, &mut result);
    result
}
fn source_bytes(source: &AnnularSourceHistory) -> Vec<u8> {
    let mut result = format!(
        "annular-source-v2;source={:?};initial-copy={:?};partition-queries={:?};closure={:?};",
        source.source, source.initial_copy, source.partition_queries, source.closure
    )
    .into_bytes();
    block(&partition_metadata::replay(&source.metadata), &mut result);
    result
}
