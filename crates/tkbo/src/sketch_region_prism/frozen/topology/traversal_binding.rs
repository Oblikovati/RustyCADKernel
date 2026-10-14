// SPDX-License-Identifier: GPL-2.0-only
//! A captured ordering is evidence only after a bijection to the original containment uses.
use super::super::{traversal::FrozenWireTraversal, *};
use super::{execution::TopologyWork, model::*, types::*};
use std::collections::BTreeSet;
pub(super) fn check<'a>(
    source: &'a FrozenPrismProjection,
    index: &mut TopologyIndex<'a>,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<()> {
    let rows = source.wire_traversals.as_deref().unwrap_or_default();
    for (position, row) in rows.iter().enumerate() {
        work.poll(
            TopologyInvariant::ClosedContainment,
            3 * index.occurrences.len() + position + 1,
        )?;
        add_row(index, row)?;
    }
    complete(index)
}
fn complete(index: &TopologyIndex<'_>) -> TopologyResult<()> {
    let wires = index.definitions.iter().filter(|key| key.kind == 5).count();
    count(
        None,
        index.traversals.len(),
        wires,
        "complete traversal evidence for every wire",
    )
}

fn add_row<'a>(index: &mut TopologyIndex<'a>, row: &'a FrozenWireTraversal) -> TopologyResult<()> {
    row_binding(index, row)?;
    if index
        .traversals
        .insert(row.occurrence.clone(), &row.coedges)
        .is_some()
    {
        return Err(invalid(
            Some(&row.occurrence),
            TopologyActual::Count(2),
            "one traversal per wire occurrence",
        ));
    }
    Ok(())
}

fn row_binding(index: &TopologyIndex<'_>, row: &FrozenWireTraversal) -> TopologyResult<()> {
    let wire = index.occurrence(&row.occurrence)?;
    let actual: BTreeSet<_> = row.coedges.iter().collect();
    let expected: BTreeSet<_> = wire.children.iter().collect();
    if definition(&wire.key)?.kind != 5 || actual.len() != row.coedges.len() || actual != expected {
        return Err(invalid(
            Some(&row.occurrence),
            TopologyActual::Count(actual.len()),
            "bijective ordered coedges covering exactly this wire's original directed edge uses",
        ));
    }
    Ok(())
}
