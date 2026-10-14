// SPDX-License-Identifier: GPL-2.0-only
//! Preserve every original parent and its relation class; sorting never deduplicates multiplicity.
use super::*;
use crate::sketch_region::{
    ArrangementRelation, OriginalMemberIdentity, PartitionOccurrenceIdentity,
};

pub(super) fn original_keys(
    request: &PrismHistoryRequest<'_, '_>,
    parents: &[OriginalMemberIdentity],
) -> HistoryCheck<Vec<FrozenOriginalKey>> {
    let mut result = parents
        .iter()
        .map(|p| {
            super::super::identities::original_key(request.publication, p)
                .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))
        })
        .collect::<HistoryCheck<Vec<_>>>()?;
    result.sort();
    Ok(result)
}
pub(super) fn parents(
    request: &PrismHistoryRequest<'_, '_>,
    preserved: &[OriginalMemberIdentity],
    modified: &[OriginalMemberIdentity],
    generated: &[OriginalMemberIdentity],
) -> HistoryCheck<[Vec<FrozenOriginalKey>; 3]> {
    Ok([
        original_keys(request, preserved)?,
        original_keys(request, modified)?,
        original_keys(request, generated)?,
    ])
}
pub(super) fn relation_parents(
    request: &PrismHistoryRequest<'_, '_>,
    relation: &ArrangementRelation,
) -> HistoryCheck<[Vec<FrozenOriginalKey>; 3]> {
    parents(
        request,
        &relation.preserved,
        &relation.modified,
        &relation.generated,
    )
}
pub(super) fn relation_rows(
    request: &PrismHistoryRequest<'_, '_>,
    bound: &BoundHistoricalContexts<'_>,
    rows: &[ArrangementRelation],
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    history_count(
        "one relation per actual bounded directed context",
        rows.len(),
        bound.actual.len(),
    )?;
    let mut result = Vec::new();
    for (index, actual) in bound.actual.iter().enumerate() {
        let row = exact_relation(rows, &actual.occurrence)?;
        result.push(PrismHistoricalRelation {
            output: bound.owned[index].key.clone(),
            parents: relation_parents(request, row)?,
        });
    }
    result.sort();
    Ok(result)
}
pub(super) fn occurrence_index(
    bound: &BoundHistoricalContexts<'_>,
    actual: &PartitionOccurrenceIdentity,
) -> HistoryCheck<usize> {
    let found: Vec<_> = bound
        .actual
        .iter()
        .enumerate()
        .filter(|(_, row)| &row.occurrence == actual)
        .map(|(i, _)| i)
        .collect();
    history_count("one actual bounded directed occurrence", found.len(), 1)?;
    Ok(found[0])
}
pub(super) fn physical_index(
    bound: &BoundHistoricalContexts<'_>,
    actual: &PartitionOccurrenceIdentity,
) -> HistoryCheck<usize> {
    let found: Vec<_> = bound
        .actual
        .iter()
        .enumerate()
        .filter(|(_, row)| same_definition(&row.occurrence, actual))
        .map(|(i, _)| i)
        .collect();
    let first = *found.first().ok_or(PrismHistoryCause::Missing {
        required: "actual historical physical definition",
    })?;
    admit_physical_name(bound, &found, first)?;
    Ok(first)
}

pub(super) fn relation_rows_subset(
    request: &PrismHistoryRequest<'_, '_>,
    bound: &BoundHistoricalContexts<'_>,
    rows: &[ArrangementRelation],
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    let mut result = Vec::new();
    for row in rows {
        let index = occurrence_index(bound, &PartitionOccurrenceIdentity::capture(&row.output))?;
        result.push(PrismHistoricalRelation {
            output: bound.owned[index].key.clone(),
            parents: relation_parents(request, row)?,
        });
    }
    Ok(observed_values::sorted(result))
}

fn exact_relation<'a>(
    rows: &'a [ArrangementRelation],
    actual: &PartitionOccurrenceIdentity,
) -> HistoryCheck<&'a ArrangementRelation> {
    let matched: Vec<_> = rows
        .iter()
        .filter(|r| PartitionOccurrenceIdentity::capture(&r.output) == *actual)
        .collect();
    history_count(
        "one exact relation for actual directed context",
        matched.len(),
        1,
    )?;
    Ok(matched[0])
}

fn admit_physical_name(
    bound: &BoundHistoricalContexts<'_>,
    found: &[usize],
    first: usize,
) -> HistoryCheck<()> {
    let member = bound.owned[first].occurrence.member;
    history_count(
        "one authored historical physical key",
        usize::from(
            found
                .iter()
                .all(|i| bound.owned[*i].occurrence.member == member),
        ),
        1,
    )?;
    Ok(())
}
