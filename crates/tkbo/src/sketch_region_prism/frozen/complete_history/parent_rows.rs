// SPDX-License-Identifier: GPL-2.0-only
//! Preserve every original parent and its relation class; sorting never deduplicates multiplicity.
use super::*;
use crate::sketch_region::{
    ArrangementRelation, OriginalMemberIdentity, PartitionOccurrenceIdentity,
};

pub(in crate::sketch_region_prism::frozen) fn original_keys(
    publication: &PrismPublicationBindings,
    parents: &[OriginalMemberIdentity],
) -> HistoryCheck<Vec<FrozenOriginalKey>> {
    let mut result = parents
        .iter()
        .map(|p| {
            super::super::identities::original_key(publication, p)
                .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))
        })
        .collect::<HistoryCheck<Vec<_>>>()?;
    result.sort();
    Ok(result)
}
pub(in crate::sketch_region_prism::frozen) fn parents(
    publication: &PrismPublicationBindings,
    preserved: &[OriginalMemberIdentity],
    modified: &[OriginalMemberIdentity],
    generated: &[OriginalMemberIdentity],
) -> HistoryCheck<[Vec<FrozenOriginalKey>; 3]> {
    Ok([
        original_keys(publication, preserved)?,
        original_keys(publication, modified)?,
        original_keys(publication, generated)?,
    ])
}
pub(in crate::sketch_region_prism::frozen) fn relation_parents(
    publication: &PrismPublicationBindings,
    relation: &ArrangementRelation,
) -> HistoryCheck<[Vec<FrozenOriginalKey>; 3]> {
    parents(
        publication,
        &relation.preserved,
        &relation.modified,
        &relation.generated,
    )
}
pub(in crate::sketch_region_prism::frozen) fn relation_rows(
    publication: &PrismPublicationBindings,
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
            parents: relation_parents(publication, row)?,
        });
    }
    result.sort();
    Ok(result)
}
pub(in crate::sketch_region_prism::frozen) fn occurrence_index(
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
pub(in crate::sketch_region_prism::frozen) fn physical_index(
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

pub(in crate::sketch_region_prism::frozen) fn relation_rows_subset(
    publication: &PrismPublicationBindings,
    bound: &BoundHistoricalContexts<'_>,
    rows: &[ArrangementRelation],
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    let mut result = Vec::new();
    for row in rows {
        let index = occurrence_index(bound, &PartitionOccurrenceIdentity::capture(&row.output))?;
        result.push(PrismHistoricalRelation {
            output: bound.owned[index].key.clone(),
            parents: relation_parents(publication, row)?,
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
