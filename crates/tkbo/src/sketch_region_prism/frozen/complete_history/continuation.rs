// SPDX-License-Identifier: GPL-2.0-only
//! Empty original deletion is concluded only after every original directed use reaches kept cap outputs.
use super::super::lineage::PrismLineageInspection;
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismOriginalContinuation {
    pub original: FrozenOriginalKey,
    pub original_use: PrismHistoryUseKey,
    pub input_copy: PrismHistoryUseKey,
    pub partition: PrismHistoryUseKey,
    pub candidate_copy: PrismHistoryUseKey,
    pub selected: PrismHistoryUseKey,
    pub complement: PrismHistoryUseKey,
    pub sweep_base: PrismHistoryUseKey,
    pub kept_first: Vec<FrozenUseKey>,
    pub kept_last: Vec<FrozenUseKey>,
}
pub(super) fn prove(
    request: &PrismHistoryRequest<'_, '_>,
    history: &PrismIntermediateHistory,
    lineage: &PrismLineageInspection,
    queries: &[PrismCompleteSweepQuery],
) -> HistoryCheck<Vec<PrismOriginalContinuation>> {
    let mut result = Vec::new();
    for query in &history.partition_queries {
        result.push(continue_original(
            request, history, lineage, queries, query,
        )?);
    }
    history_count(
        "all original directed uses proven to retained body",
        result.len(),
        request.publication.originals.len(),
    )?;
    let originals = result
        .iter()
        .map(|r| &r.original)
        .collect::<std::collections::BTreeSet<_>>();
    history_count(
        "distinct original use continuation census",
        originals.len(),
        result.len(),
    )?;
    Ok(observed_values::sorted(result))
}
fn continue_original(
    request: &PrismHistoryRequest<'_, '_>,
    history: &PrismIntermediateHistory,
    lineage: &PrismLineageInspection,
    queries: &[PrismCompleteSweepQuery],
    query: &PrismPartitionQuery,
) -> HistoryCheck<PrismOriginalContinuation> {
    let original_use = original_use(request, history, &query.original)?;
    let input_copy = copied_use(&history.initial_copy, &original_use)?;
    let candidate_copy = copied_use(&history.candidate_copy, &query.continued)?;
    let generated = generated_use(history, &candidate_copy)?;
    let sweep_base = sweep_lineage::copied_use(lineage, &generated.selected)?;
    let sweep = sweep_query(queries, &sweep_base)?;
    let (kept_first, kept_last) = kept_outputs(sweep, &query.original)?;
    Ok(PrismOriginalContinuation {
        original: query.original.clone(),
        original_use,
        input_copy,
        partition: query.continued.clone(),
        candidate_copy,
        selected: generated.selected.clone(),
        complement: generated.complement.clone(),
        sweep_base,
        kept_first,
        kept_last,
    })
}
fn original_use(
    request: &PrismHistoryRequest<'_, '_>,
    history: &PrismIntermediateHistory,
    original: &FrozenOriginalKey,
) -> HistoryCheck<PrismHistoryUseKey> {
    let matches: Vec<_> = history
        .contexts
        .iter()
        .filter(|r| {
            r.occurrence.member.stage == PrismHistoryStage::Original
                && r.occurrence.member.member == original.member
                && r.key
                    .0
                    .last()
                    .is_some_and(|(_, id)| id == &original.directed_use)
        })
        .collect();
    history_count("one explicitly named original context", matches.len(), 1)?;
    history_count(
        "original continuation belongs to authored operation",
        usize::from(matches[0].occurrence.member.operation == request.publication.operation),
        1,
    )?;
    Ok(matches[0].key.clone())
}
fn copied_use(
    rows: &[PrismHistoricalCopyQuery],
    source: &PrismHistoryUseKey,
) -> HistoryCheck<PrismHistoryUseKey> {
    let found: Vec<_> = rows.iter().filter(|q| &q.source == source).collect();
    history_count(
        "one actual directed copy link in original continuation",
        found.len(),
        1,
    )?;
    Ok(found[0].copied.clone())
}
fn generated_use<'a>(
    history: &'a PrismIntermediateHistory,
    copied: &PrismHistoryUseKey,
) -> HistoryCheck<&'a PrismGeneratedTrace> {
    let rows: Vec<_> = history
        .candidates
        .directed
        .iter()
        .filter(|r| &r.copied == copied)
        .collect();
    history_count(
        "one actual generation continuation in both complementary regions",
        rows.len(),
        1,
    )?;
    Ok(rows[0])
}
fn sweep_query<'a>(
    rows: &'a [PrismCompleteSweepQuery],
    source: &PrismHistoryUseKey,
) -> HistoryCheck<&'a PrismCompleteSweepQuery> {
    let rows: Vec<_> = rows.iter().filter(|r| &r.copied == source).collect();
    history_count(
        "one exhaustive sweep query for original continuation",
        rows.len(),
        1,
    )?;
    Ok(rows[0])
}
fn kept_outputs(
    query: &PrismCompleteSweepQuery,
    original: &FrozenOriginalKey,
) -> HistoryCheck<(Vec<FrozenUseKey>, Vec<FrozenUseKey>)> {
    history_count(
        "original continuation is not copied-intermediate retirement",
        usize::from(query.retired),
        0,
    )?;
    history_count(
        "original continuation retains exact source parent",
        usize::from(query.parents.iter().flatten().any(|p| p == original)),
        1,
    )?;
    let first = query.first.as_ref().ok_or(PrismHistoryCause::Missing {
        required: "actual first cap continuation",
    })?;
    let last = query.last.as_ref().ok_or(PrismHistoryCause::Missing {
        required: "actual last cap continuation",
    })?;
    Ok((first.outputs.clone(), last.outputs.clone()))
}
