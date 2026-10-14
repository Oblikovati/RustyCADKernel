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
    let mut matches = Vec::new();
    for (index, row) in target.actual.iter().enumerate() {
        if matches_trace(source, row)? {
            matches.push(index);
        }
    }
    history_count(
        "unique exact directed generated correspondence",
        matches.len(),
        1,
    )?;
    Ok(target.owned[matches[0]].key.clone())
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
