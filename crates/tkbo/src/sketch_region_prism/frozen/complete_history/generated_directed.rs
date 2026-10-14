// SPDX-License-Identifier: GPL-2.0-only
//! Relative endpoint orientation is transported under its actual edge, never inferred from coordinates.
use super::*;
use crate::sketch_region::RegionObservedContext;
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn project(
    copied: &BoundHistoricalContexts<'_>,
    selected: &BoundHistoricalContexts<'_>,
    complement: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<Vec<PrismGeneratedTrace>> {
    let mut result = Vec::new();
    for (index, source) in copied
        .actual
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kind != ShapeEnum::Compound)
    {
        result.push(PrismGeneratedTrace {
            copied: copied.owned[index].key.clone(),
            selected: unique_trace(source, selected)?,
            complement: unique_trace(source, complement)?,
        });
    }
    history_count(
        "all copied edge and both closure uses continue into both regions",
        result.len(),
        3,
    )?;
    admit_bijection(&result)?;
    Ok(observed_values::sorted(result))
}
fn unique_trace(
    source: &RegionObservedContext,
    target: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<PrismHistoryUseKey> {
    let index = resolve_generated_context(source, target.actual)?;
    Ok(target.owned[index].key.clone())
}
/// Lookup actual relative endpoint correspondence under its retained edge ancestry.
/// Example: `resolve_generated_context(source, targets)`; target order never authors a name.
pub(in crate::sketch_region_prism) fn resolve_generated_context(
    source: &RegionObservedContext,
    target: &[RegionObservedContext],
) -> HistoryCheck<usize> {
    let matches = generated_context_matches(source, target)?;
    history_count(
        "unique exact directed generated correspondence",
        matches.len(),
        1,
    )?;
    Ok(matches[0])
}
/// Every exact directed generated correspondence in one candidate; may be empty.
/// Example: `generated_context_matches(source, targets)?.len()` counts candidates reached.
pub(in crate::sketch_region_prism) fn generated_context_matches(
    source: &RegionObservedContext,
    target: &[RegionObservedContext],
) -> HistoryCheck<Vec<usize>> {
    let mut matches = Vec::new();
    for (index, row) in target.iter().enumerate() {
        if matches_trace(source, row)? {
            matches.push(index);
        }
    }
    Ok(matches)
}
fn matches_trace(
    source: &RegionObservedContext,
    target: &RegionObservedContext,
) -> HistoryCheck<bool> {
    if source.kind != target.kind || !same_definition(&source.occurrence, &target.occurrence) {
        return Ok(false);
    }
    if source.kind == ShapeEnum::Edge {
        return Ok(source.ancestors.len() == 1 && target.ancestors.len() == 2);
    }
    if source.kind != ShapeEnum::Vertex
        || source.ancestors.len() != 2
        || target.ancestors.len() != 3
    {
        return Ok(false);
    }
    matches_endpoint_trace(source, target)
}
fn admit_bijection(rows: &[PrismGeneratedTrace]) -> HistoryCheck<()> {
    let selected = rows
        .iter()
        .map(|r| &r.selected)
        .collect::<std::collections::BTreeSet<_>>();
    let complement = rows
        .iter()
        .map(|r| &r.complement)
        .collect::<std::collections::BTreeSet<_>>();
    history_count(
        "selected generated-use bijection",
        selected.len(),
        rows.len(),
    )?;
    history_count(
        "complement generated-use bijection",
        complement.len(),
        rows.len(),
    )
}

fn matches_endpoint_trace(
    source: &RegionObservedContext,
    target: &RegionObservedContext,
) -> HistoryCheck<bool> {
    let edge_source = &source.ancestors[1];
    let edge_target = &target.ancestors[2];
    if !same_definition(edge_source, edge_target) {
        return Ok(false);
    }
    let local_source = census::forward_reversed(source.occurrence.orientation)?
        ^ census::forward_reversed(edge_source.orientation)?;
    let local_target = census::forward_reversed(target.occurrence.orientation)?
        ^ census::forward_reversed(edge_target.orientation)?;
    Ok(local_source == local_target)
}
