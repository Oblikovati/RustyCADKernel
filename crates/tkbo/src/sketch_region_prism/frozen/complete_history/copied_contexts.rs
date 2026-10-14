// SPDX-License-Identifier: GPL-2.0-only
//! Actual Modified observations at every ancestor establish unique directed correspondence.
use super::*;
use crate::sketch_region::{RegionContextCopyQuery, RegionCopyContexts, RegionObservedContext};
use std::collections::BTreeSet;

pub(super) fn project_copier_queries(
    observed: &RegionCopyContexts,
    source: &BoundHistoricalContexts<'_>,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<Vec<PrismHistoricalCopyQuery>> {
    admit_query_censuses(observed, source, copied)?;
    let mut result = Vec::new();
    for (index, actual) in source.actual.iter().enumerate() {
        let target = resolve_native_copy_context(observed, actual, copied.actual)
            .map_err(|cause| cause.at_use(source.owned[index].key.clone()))?;
        result.push(query_row(
            observed,
            actual,
            &source.owned[index],
            &copied.owned[target],
        )?);
    }
    let consumed: BTreeSet<_> = result.iter().map(|r| &r.copied).collect();
    history_count(
        "complete one-to-one directed copier correspondence",
        consumed.len(),
        copied.actual.len(),
    )?;
    result.sort();
    Ok(result)
}
fn query_row(
    observed: &RegionCopyContexts,
    actual: &RegionObservedContext,
    source: &PrismHistoricalContext,
    copied: &PrismHistoricalContext,
) -> HistoryCheck<PrismHistoricalCopyQuery> {
    let query = query_for(observed, actual)?;
    let images = query
        .images
        .as_ref()
        .map_err(|e| PrismHistoryCause::Native(e.clone()))?;
    let native_images = images
        .iter()
        .map(|image| context_names::historical_occurrence(copied.occurrence.member, image))
        .collect::<HistoryCheck<_>>()?;
    Ok(PrismHistoricalCopyQuery {
        source: source.key.clone(),
        native_images,
        copied: copied.key.clone(),
    })
}
/// Resolve one local target index from actual native images at every directed ancestor.
/// The index is lookup-only; example: `resolve_native_copy_context(rows, source, targets)`.
pub(in crate::sketch_region_prism) fn resolve_native_copy_context(
    observed: &RegionCopyContexts,
    source: &RegionObservedContext,
    copied: &[RegionObservedContext],
) -> HistoryCheck<usize> {
    let mut found = Vec::new();
    for (index, target) in copied.iter().enumerate() {
        if mapped_context(observed, source, target)? {
            found.push(index);
        }
    }
    history_count(
        "unique copied use from actual mapped ancestors, placement and orientation",
        found.len(),
        1,
    )?;
    Ok(found[0])
}
fn mapped_context(
    observed: &RegionCopyContexts,
    source: &RegionObservedContext,
    copied: &RegionObservedContext,
) -> HistoryCheck<bool> {
    if source.ancestors.len() != copied.ancestors.len() {
        return Ok(false);
    }
    for depth in 0..=source.ancestors.len() {
        let ancestor = context_names::ancestor_at(&observed.source, source, depth)?;
        let target = if depth == copied.ancestors.len() {
            &copied.occurrence
        } else {
            &copied.ancestors[depth]
        };
        if !mapped_occurrence(query_for(observed, ancestor)?, target)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn mapped_occurrence(
    query: &RegionContextCopyQuery,
    copied: &crate::sketch_region::PartitionOccurrenceIdentity,
) -> HistoryCheck<bool> {
    let images = query
        .images
        .as_ref()
        .map_err(|e| PrismHistoryCause::Native(e.clone()))?;
    history_count(
        "one actual Modified image per bounded copied member",
        images.len(),
        1,
    )?;
    Ok(same_definition(&images[0], copied)
        && query.source.occurrence.location == copied.location
        && query.source.occurrence.orientation == copied.orientation)
}
fn query_for<'a>(
    observed: &'a RegionCopyContexts,
    actual: &RegionObservedContext,
) -> HistoryCheck<&'a RegionContextCopyQuery> {
    let found: Vec<_> = observed
        .queries
        .iter()
        .filter(|q| &q.source == actual)
        .collect();
    history_count(
        "exactly one Modified observation for actual source context",
        found.len(),
        1,
    )?;
    Ok(found[0])
}

fn admit_query_censuses(
    observed: &RegionCopyContexts,
    source: &BoundHistoricalContexts<'_>,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<()> {
    history_count(
        "all directed copier query outcomes",
        observed.queries.len(),
        source.actual.len(),
    )?;
    history_count(
        "same retained source contexts",
        usize::from(observed.source == source.actual),
        1,
    )?;
    history_count(
        "same retained copied contexts",
        usize::from(observed.copied == copied.actual),
        1,
    )?;
    Ok(())
}
