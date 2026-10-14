// SPDX-License-Identifier: GPL-2.0-only
//! Version one proves unchanged native partition traces from complete actual member queries.
use super::*;
use crate::sketch_region::{
    PartitionMemberQueries, PartitionMemberQuery, PartitionOccurrenceIdentity,
    RegionObservedContext,
};
use stages::BoundHistoryStage;

pub(super) fn project_partition_queries(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
) -> HistoryCheck<Vec<PrismPartitionQuery>> {
    let observed = query_rows(request)?;
    let original = stages::stage(stages, PrismHistoryStage::Original)?;
    let copied = stages::stage(stages, PrismHistoryStage::InputCopy)?;
    let partition = stages::stage(stages, PrismHistoryStage::Partition)?;
    history_count(
        "exhaustive native partition query census",
        observed.len(),
        original.actual.len(),
    )?;
    let mut result = Vec::new();
    for (index, actual) in original.actual.iter().enumerate() {
        let query = original_query(request, observed, actual)?;
        result.push(
            project_query(request, query, actual, copied, partition)
                .map_err(|cause| cause.at_use(original.owned[index].key.clone()))?,
        );
    }
    result.sort();
    Ok(result)
}
fn query_rows<'a>(
    request: &PrismHistoryRequest<'a, '_>,
) -> HistoryCheck<&'a [PartitionMemberQuery]> {
    let PartitionMemberQueries::Captured {
        history_requested,
        history_available,
        rows,
    } = &operands::partition(request)?.member_queries
    else {
        return Err(PrismHistoryCause::Missing {
            required: "exhaustive native partition member queries",
        });
    };
    admit_partition_history_availability(*history_requested, *history_available)?;
    Ok(rows)
}
fn project_query(
    request: &PrismHistoryRequest<'_, '_>,
    query: &PartitionMemberQuery,
    original: &RegionObservedContext,
    copied: &BoundHistoricalContexts<'_>,
    partition: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<PrismPartitionQuery> {
    admit_unchanged_partition_query(query, original)?;
    let copied_index = parent_rows::occurrence_index(copied, &query.copied)?;
    bind_initial_query(request, query, original)?;
    let continued =
        resolve_native_partition_context(&copied.actual[copied_index], partition.actual)?;
    Ok(PrismPartitionQuery {
        original: super::super::identities::original_key(request.publication, &query.original)
            .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))?,
        copied: copied.owned[copied_index].occurrence.clone(),
        kind: original.kind as u8,
        supported: true,
        modified: native_images(&query.modified, partition)?,
        generated: native_images(&query.generated, partition)?,
        deleted: *native_result(&query.deleted)?,
        continued: partition.owned[continued].key.clone(),
    })
}
pub(in crate::sketch_region_prism) fn admit_unchanged_partition_query(
    query: &PartitionMemberQuery,
    original: &RegionObservedContext,
) -> HistoryCheck<()> {
    history_count(
        "native query kind equals original census",
        usize::from(*native_result(&query.kind)? == original.kind),
        1,
    )?;
    history_count(
        "supported native original member history",
        usize::from(*native_result(&query.supported)?),
        1,
    )?;
    admit_unchanged_answers(query)
}
fn bind_initial_query(
    request: &PrismHistoryRequest<'_, '_>,
    query: &PartitionMemberQuery,
    original: &RegionObservedContext,
) -> HistoryCheck<()> {
    let image = initial_query_image(request, original)?;
    history_count(
        "partition query targets its actual initial copied image",
        usize::from(
            same_definition(image, &query.copied)
                && query.copied.orientation == original.occurrence.orientation,
        ),
        1,
    )
}
/// Lookup the unique unchanged trace beneath the actual partition root.
/// Example: `resolve_native_partition_context(source, targets)`; its index is not identity.
pub(in crate::sketch_region_prism) fn resolve_native_partition_context(
    source: &RegionObservedContext,
    partition: &[RegionObservedContext],
) -> HistoryCheck<usize> {
    let found: Vec<_> = partition
        .iter()
        .enumerate()
        .filter(|(_, target)| same_partition_tail(source, target))
        .map(|(i, _)| i)
        .collect();
    history_count(
        "one exact unchanged directed trace under the partition container",
        found.len(),
        1,
    )?;
    Ok(found[0])
}
fn same_partition_tail(source: &RegionObservedContext, target: &RegionObservedContext) -> bool {
    target.ancestors.len() == source.ancestors.len() + 1
        && target.occurrence == source.occurrence
        && target.ancestors[1..] == source.ancestors
}
pub(in crate::sketch_region_prism::frozen) fn native_images(
    result: &Result<
        Vec<PartitionOccurrenceIdentity>,
        std::sync::Arc<rustycad_tkernel::standard::Failure>,
    >,
    partition: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<Vec<PrismHistoricalOccurrence>> {
    native_result(result)?
        .iter()
        .map(|actual| {
            let index = parent_rows::physical_index(partition, actual)?;
            context_names::historical_occurrence(partition.owned[index].occurrence.member, actual)
        })
        .collect()
}
pub(in crate::sketch_region_prism::frozen) fn native_result<T>(
    result: &Result<T, std::sync::Arc<rustycad_tkernel::standard::Failure>>,
) -> HistoryCheck<&T> {
    result
        .as_ref()
        .map_err(|cause| PrismHistoryCause::Native(cause.clone()))
}

fn original_query<'a>(
    request: &PrismHistoryRequest<'_, '_>,
    observed: &'a [PartitionMemberQuery],
    actual: &RegionObservedContext,
) -> HistoryCheck<&'a PartitionMemberQuery> {
    let identity = stage_names::original_identity(request, &actual.occurrence)?;
    let rows: Vec<_> = observed.iter().filter(|q| q.original == identity).collect();
    history_count(
        "one actual native partition query for each original use",
        rows.len(),
        1,
    )?;
    Ok(rows[0])
}

pub(in crate::sketch_region_prism) fn admit_partition_history_availability(
    history_requested: bool,
    history_available: bool,
) -> HistoryCheck<()> {
    history_count(
        "history filling requested",
        usize::from(history_requested),
        1,
    )?;
    history_count(
        "actual filled native history available",
        usize::from(history_available),
        1,
    )?;
    Ok(())
}

fn admit_unchanged_answers(query: &PartitionMemberQuery) -> HistoryCheck<()> {
    history_count(
        "v1 unchanged Jordan partition Modified answers",
        native_result(&query.modified)?.len(),
        0,
    )?;
    history_count(
        "v1 unchanged Jordan partition Generated answers",
        native_result(&query.generated)?.len(),
        0,
    )?;
    history_count(
        "v1 continued original member is not deleted",
        usize::from(*native_result(&query.deleted)?),
        0,
    )
}

fn initial_query_image<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    original: &RegionObservedContext,
) -> HistoryCheck<&'a PartitionOccurrenceIdentity> {
    let initial = operands::copied_contexts(
        &operands::partition(request)?.input_contexts[0].observation,
        PrismHistoryStage::InputCopy,
    )?;
    let rows: Vec<_> = initial
        .queries
        .iter()
        .filter(|q| q.source == *original)
        .collect();
    history_count(
        "one actual initial copier observation for partition query",
        rows.len(),
        1,
    )?;
    let images = unique_initial_image(rows[0])?;
    Ok(&images[0])
}

fn unique_initial_image(
    query: &crate::sketch_region::RegionContextCopyQuery,
) -> HistoryCheck<&[PartitionOccurrenceIdentity]> {
    let images = native_result(&query.images)?;
    history_count(
        "one actual initial copier image for partition query",
        images.len(),
        1,
    )?;
    Ok(images)
}
