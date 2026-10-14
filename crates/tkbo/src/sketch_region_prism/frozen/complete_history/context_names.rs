// SPDX-License-Identifier: GPL-2.0-only
//! Existing frozen identity/placement helpers remain the source of scalar naming semantics.
use super::*;
use crate::sketch_region::{PartitionOccurrenceIdentity, RegionObservedContext};

pub(in crate::sketch_region_prism::frozen) fn historical_member(
    operation: AuthoredPrismId,
    names: &PrismHistoricalStageBindings,
    actual: &RegionObservedContext,
) -> HistoryCheck<PrismHistoryMemberKey> {
    let member = authored_member(names, actual)?;
    Ok(PrismHistoryMemberKey {
        operation,
        stage: names.stage,
        member,
        kind: actual.kind as u8,
    })
}
pub(in crate::sketch_region_prism::frozen) fn historical_occurrence(
    member: PrismHistoryMemberKey,
    actual: &PartitionOccurrenceIdentity,
) -> HistoryCheck<PrismHistoricalOccurrence> {
    let placement = super::super::history::freeze_placement(&actual.location)
        .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))?;
    let values = placement
        .linear
        .into_iter()
        .flatten()
        .chain(placement.origin)
        .chain([placement.scale]);
    let mut placement_bits = [0; 13];
    for (slot, value) in placement_bits.iter_mut().zip(values) {
        *slot = value.to_bits();
    }
    Ok(PrismHistoricalOccurrence {
        member,
        orientation: actual.orientation as u8,
        placement_bits,
    })
}
pub(in crate::sketch_region_prism::frozen) fn historical_use(
    names: &PrismHistoricalStageBindings,
    actual: &RegionObservedContext,
) -> HistoryCheck<AuthoredPrismId> {
    let matches: Vec<_> = names.uses.iter().filter(|b| &b.actual == actual).collect();
    history_count(
        "exactly one authored historical directed context",
        matches.len(),
        1,
    )?;
    let name = matches[0].authored_use;
    history_count(
        "nonzero authored historical use",
        usize::from(name.0 != 0),
        1,
    )?;
    Ok(name)
}
pub(in crate::sketch_region_prism::frozen) fn ancestor_at<'a>(
    rows: &'a [RegionObservedContext],
    context: &RegionObservedContext,
    depth: usize,
) -> HistoryCheck<&'a RegionObservedContext> {
    let rows: Vec<_> = rows
        .iter()
        .filter(|r| r.child_slots == context.child_slots[..depth])
        .collect();
    history_count("one observed ancestor at each directed path", rows.len(), 1)?;
    if depth < context.ancestors.len() {
        history_count(
            "exact observed ancestor occurrence",
            usize::from(rows[0].occurrence == context.ancestors[depth]),
            1,
        )?;
    }
    Ok(rows[0])
}

fn authored_member(
    names: &PrismHistoricalStageBindings,
    actual: &RegionObservedContext,
) -> HistoryCheck<AuthoredPrismId> {
    let matches: Vec<_> = names
        .members
        .iter()
        .filter(|b| same_definition(&b.actual, &actual.occurrence))
        .collect();
    history_count(
        "exactly one authored historical definition",
        matches.len(),
        1,
    )?;
    let member = matches[0].member;
    admit_member_name(names, member)?;
    Ok(member)
}

fn admit_member_name(
    names: &PrismHistoricalStageBindings,
    member: AuthoredPrismId,
) -> HistoryCheck<()> {
    history_count(
        "nonzero authored historical member",
        usize::from(member.0 != 0),
        1,
    )?;
    history_count(
        "one physical definition per historical member",
        names.members.iter().filter(|b| b.member == member).count(),
        1,
    )?;
    Ok(())
}
