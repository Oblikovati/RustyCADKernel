// SPDX-License-Identifier: GPL-2.0-only
//! Partition parents follow each original's own native continuation, never same-kind grouping.
use super::super::complete_history::{observed_values, parent_rows, relations};
use super::sources::AnnularSourceHistory;
use super::*;
use crate::sketch_region::ArrangementCompletion;

/// Bind every actual partition relation to the continuation-derived original parent classes.
pub(super) fn capture(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    partition: &BoundHistoricalContexts<'_>,
    sources: &[AnnularSourceHistory],
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    let ArrangementCompletion::Arranged(owned) =
        &request.region.bridge.mapped.arrangement.completion
    else {
        return Err(PrismHistoryCause::Missing {
            required: "actual arranged partition relations",
        });
    };
    history_count(
        "bounded original partition deletion is empty",
        owned.deleted.len(),
        0,
    )?;
    let rows = parent_rows::relation_rows(request.publication, partition, &owned.relations)?;
    let queries: Vec<_> = sources.iter().flat_map(|s| &s.partition_queries).collect();
    for row in &rows {
        let expected = expected_parents(partition, row, &queries)?;
        relations::same_parents(PrismHistoryStage::Partition, &row.parents, &expected)
            .map_err(|cause| cause.at_use(row.output.clone()))?;
    }
    Ok(rows)
}
/// The root container is generated from every original; each member is modified from exactly the
/// originals whose actual continuation reaches the same physical partition member.
fn expected_parents(
    partition: &BoundHistoricalContexts<'_>,
    row: &PrismHistoricalRelation,
    queries: &[&PrismPartitionQuery],
) -> HistoryCheck<[Vec<FrozenOriginalKey>; 3]> {
    let index = context_index(partition, &row.output)?;
    if partition.actual[index].child_slots.is_empty() {
        let all = queries.iter().map(|q| q.original.clone()).collect();
        return Ok([vec![], vec![], observed_values::sorted(all)]);
    }
    let member = partition.owned[index].occurrence.member;
    let mut parents = Vec::new();
    for query in queries {
        let target = context_index(partition, &query.continued)?;
        if partition.owned[target].occurrence.member == member {
            parents.push(query.original.clone());
        }
    }
    Ok([vec![], observed_values::sorted(parents), vec![]])
}
/// Lookup-only index of one bound key.
pub(super) fn context_index(
    bound: &BoundHistoricalContexts<'_>,
    key: &PrismHistoryUseKey,
) -> HistoryCheck<usize> {
    let rows: Vec<_> = bound
        .owned
        .iter()
        .enumerate()
        .filter(|(_, c)| &c.key == key)
        .map(|(i, _)| i)
        .collect();
    history_count("one bound context for history key", rows.len(), 1)?;
    Ok(rows[0])
}
