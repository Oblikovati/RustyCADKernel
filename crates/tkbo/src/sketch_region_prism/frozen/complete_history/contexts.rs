// SPDX-License-Identifier: GPL-2.0-only
//! Complete authored bijection labels actual contexts; paths are lookup-only operands.
use super::*;
use crate::sketch_region::RegionObservedContext;
use std::collections::BTreeSet;

pub(super) fn bind_historical_contexts<'a>(
    operation: AuthoredPrismId,
    names: &PrismHistoricalStageBindings,
    rows: &'a [RegionObservedContext],
    limits: PrismHistoryLimits,
) -> HistoryCheck<BoundHistoricalContexts<'a>> {
    limits.admit().map_err(PrismHistoryCause::Bounds)?;
    history_count(
        "nonzero operation identity",
        usize::from(operation.0 != 0),
        1,
    )?;
    history_count(
        "nonempty historical context census",
        usize::from(!rows.is_empty()),
        1,
    )?;
    limits
        .count("historical uses", rows.len(), limits.maximum_uses)
        .map_err(PrismHistoryCause::Bounds)?;
    history_count(
        "one authored binding per historical use",
        names.uses.len(),
        rows.len(),
    )?;
    let mut owned = Vec::new();
    for row in rows {
        owned.push(bind_context(operation, names, rows, row, limits)?);
    }
    check_context_bijection(names, &owned)?;
    capture_children(rows, &mut owned);
    Ok(BoundHistoricalContexts {
        actual: rows,
        owned,
    })
}
fn bind_context(
    operation: AuthoredPrismId,
    names: &PrismHistoricalStageBindings,
    rows: &[RegionObservedContext],
    row: &RegionObservedContext,
    limits: PrismHistoryLimits,
) -> HistoryCheck<PrismHistoricalContext> {
    check_context(row, limits)?;
    let member = context_names::historical_member(operation, names, row)?;
    let mut key = Vec::new();
    for depth in 0..=row.child_slots.len() {
        let ancestor = context_names::ancestor_at(rows, row, depth)?;
        key.push((
            context_names::historical_member(operation, names, ancestor)?,
            context_names::historical_use(names, ancestor)?,
        ));
    }
    Ok(PrismHistoricalContext {
        key: PrismHistoryUseKey(key),
        occurrence: context_names::historical_occurrence(member, &row.occurrence)?,
        children: Vec::new(),
    })
}
fn check_context(row: &RegionObservedContext, limits: PrismHistoryLimits) -> HistoryCheck<()> {
    history_count(
        "one ancestor per child slot",
        row.ancestors.len(),
        row.child_slots.len(),
    )?;
    history_count(
        "non-null observed historical member",
        usize::from(row.occurrence.topology_address != 0),
        1,
    )?;
    limits
        .count(
            "historical ancestor depth",
            row.ancestors.len(),
            limits.maximum_depth,
        )
        .map_err(PrismHistoryCause::Bounds)?;
    for (depth, ancestor) in row.ancestors.iter().enumerate() {
        if ancestor.topology_address == row.occurrence.topology_address {
            return Err(PrismHistoryCause::Bounds(
                HistoryBoundCause::AncestorCycle {
                    ancestor_depth: depth,
                    repeated_depth: row.ancestors.len(),
                },
            ));
        }
    }
    Ok(())
}
fn check_context_bijection(
    names: &PrismHistoricalStageBindings,
    owned: &[PrismHistoricalContext],
) -> HistoryCheck<()> {
    let members: BTreeSet<_> = owned.iter().map(|row| row.occurrence.member).collect();
    let uses: BTreeSet<_> = owned.iter().map(|row| &row.key).collect();
    history_count(
        "complete historical member binding, no extras",
        members.len(),
        names.members.len(),
    )?;
    history_count(
        "unique full historical directed keys",
        uses.len(),
        owned.len(),
    )
}
fn capture_children(rows: &[RegionObservedContext], owned: &mut [PrismHistoricalContext]) {
    let keys: Vec<_> = owned.iter().map(|row| row.key.clone()).collect();
    for (actual, parent) in rows.iter().zip(owned) {
        let mut children: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row.child_slots.len() == actual.child_slots.len() + 1
                    && row.child_slots.starts_with(&actual.child_slots)
            })
            .collect();
        children.sort_by_key(|(_, row)| row.child_slots.last().copied());
        parent.children = children
            .iter()
            .map(|(index, _)| keys[*index].clone())
            .collect();
    }
}
