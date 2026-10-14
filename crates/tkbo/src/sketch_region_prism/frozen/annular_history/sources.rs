// SPDX-License-Identifier: GPL-2.0-only
//! Each original use reaches the partition only through its own source's actual copied image.
use super::super::complete_history::observed_values::sorted;
use super::super::complete_history::{copied_contexts, parent_rows, partition_queries};
use super::*;
use crate::sketch_region::{PartitionMemberQueries, PartitionMemberQuery, RegionCopyContexts};

/// Source-specific initial copy and native partition answers; `history.sources()[0].partition_queries`.
pub(crate) struct AnnularSourceHistory {
    pub source: RegionSourceId,
    pub initial_copy: Vec<PrismHistoricalCopyQuery>,
    pub partition_queries: Vec<PrismPartitionQuery>,
    pub metadata: PrismPartitionMetadata,
    pub closure: Vec<AnnularClosureWitness>,
}
/// Native closure endpoint from the sealed two-loop source proof; `witness.parameter_bits`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnnularClosureWitness {
    pub copied: bool,
    pub parameter_bits: u64,
    pub orientation: u8,
}
/// One source's bound scopes and retained initial copier census.
struct SourceScopes<'s, 'a> {
    source: RegionSourceId,
    initial: &'a RegionCopyContexts,
    original: &'s BoundHistoricalContexts<'a>,
    copied: &'s BoundHistoricalContexts<'a>,
    partition: &'s BoundHistoricalContexts<'a>,
}
/// Project both authored sources and require the exhaustive native query census.
pub(super) fn capture(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
) -> HistoryCheck<Vec<AnnularSourceHistory>> {
    let observed = query_rows(request)?;
    let mut result = Vec::new();
    for source in super::scopes::source_roles(request) {
        result.push(source_history(request, scopes, observed, source)?);
    }
    let answered: usize = result.iter().map(|r| r.partition_queries.len()).sum();
    let required = "exhaustive native partition query census";
    history_count(required, observed.len(), answered)?;
    Ok(result)
}
fn source_history(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
    observed: &[PartitionMemberQuery],
    source: RegionSourceId,
) -> HistoryCheck<AnnularSourceHistory> {
    let bound = SourceScopes {
        source,
        initial: super::scopes::initial_contexts(request, source)?,
        original: bound_scope(scopes, AnnularHistoryScope::Original(source))?,
        copied: bound_scope(scopes, AnnularHistoryScope::InputCopy(source))?,
        partition: bound_scope(scopes, AnnularHistoryScope::Partition)?,
    };
    let initial_copy =
        copied_contexts::project_copier_queries(bound.initial, bound.original, bound.copied)?;
    let metadata = super::metadata::capture(request, source, bound.original, bound.copied)?;
    Ok(AnnularSourceHistory {
        source,
        initial_copy,
        partition_queries: partition_answers(request, observed, &bound)?,
        closure: super::metadata::closure(request, source, &metadata)?,
        metadata,
    })
}
fn partition_answers(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    observed: &[PartitionMemberQuery],
    bound: &SourceScopes<'_, '_>,
) -> HistoryCheck<Vec<PrismPartitionQuery>> {
    let mut result = Vec::new();
    for (index, actual) in bound.original.actual.iter().enumerate() {
        let query = original_query(observed, bound.source, actual)?;
        let projected = project_query(request, bound, query, actual);
        let key = &bound.original.owned[index].key;
        result.push(projected.map_err(|cause| cause.at_use(key.clone()))?);
    }
    Ok(sorted(result))
}
/// Borrow one bound scope by its semantic identity.
pub(super) fn bound_scope<'a, 'b>(
    scopes: &'a [AnnularBoundScope<'b>],
    scope: AnnularHistoryScope,
) -> HistoryCheck<&'a BoundHistoricalContexts<'b>> {
    let rows: Vec<_> = scopes.iter().filter(|s| s.scope == scope).collect();
    history_count("one bound semantic history scope", rows.len(), 1)?;
    Ok(&rows[0].contexts)
}
fn query_rows<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
) -> HistoryCheck<&'a [PartitionMemberQuery]> {
    let PartitionMemberQueries::Captured {
        history_requested,
        history_available,
        rows,
    } = &super::scopes::partition_snapshot(request)?.member_queries
    else {
        return Err(PrismHistoryCause::Missing {
            required: "exhaustive native partition member queries",
        });
    };
    admit_partition_history_availability(*history_requested, *history_available)?;
    Ok(rows)
}
fn original_query<'a>(
    observed: &'a [PartitionMemberQuery],
    source: RegionSourceId,
    actual: &RegionObservedContext,
) -> HistoryCheck<&'a PartitionMemberQuery> {
    let identity = super::names::original_identity(source, &actual.occurrence);
    let rows: Vec<_> = observed.iter().filter(|q| q.original == identity).collect();
    let required = "one actual native partition query for each original use";
    history_count(required, rows.len(), 1)?;
    Ok(rows[0])
}
fn project_query(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    bound: &SourceScopes<'_, '_>,
    query: &PartitionMemberQuery,
    original: &RegionObservedContext,
) -> HistoryCheck<PrismPartitionQuery> {
    admit_unchanged_partition_query(query, original)?;
    let (copied, partition) = (bound.copied, bound.partition);
    let copied_index = parent_rows::occurrence_index(copied, &query.copied)?;
    bind_initial_image(bound.initial, query, original)?;
    let source = &copied.actual[copied_index];
    let continued = resolve_native_partition_context(source, partition.actual)?;
    let key = super::super::identities::original_key(request.publication, &query.original);
    Ok(PrismPartitionQuery {
        original: key.map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))?,
        copied: copied.owned[copied_index].occurrence.clone(),
        kind: original.kind as u8,
        supported: true,
        modified: partition_queries::native_images(&query.modified, partition)?,
        generated: partition_queries::native_images(&query.generated, partition)?,
        deleted: *partition_queries::native_result(&query.deleted)?,
        continued: partition.owned[continued].key.clone(),
    })
}
/// The query must target this source's own actual initial copier image, never another source's.
fn bind_initial_image(
    initial: &RegionCopyContexts,
    query: &PartitionMemberQuery,
    original: &RegionObservedContext,
) -> HistoryCheck<()> {
    let rows: Vec<_> = initial
        .queries
        .iter()
        .filter(|q| q.source == *original)
        .collect();
    let required = "one actual initial copier observation for partition query";
    history_count(required, rows.len(), 1)?;
    let images = partition_queries::native_result(&rows[0].images)?;
    let required = "one actual initial copier image for partition query";
    history_count(required, images.len(), 1)?;
    let same = same_definition(&images[0], &query.copied)
        && query.copied.orientation == original.occurrence.orientation;
    holds(
        "partition query targets its own source's initial copied image",
        same,
    )
}
