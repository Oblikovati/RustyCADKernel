// SPDX-License-Identifier: GPL-2.0-only
//! Bind actual generated rows to semantic selected/complement stages before removing temporary indices.
use super::*;
use crate::sketch_region_candidates::{
    GeneratedAreaSnapshot, GeneratedMemberSnapshot, GeneratedTopologyObservation,
};
pub(super) fn areas<'a>(
    request: &PrismHistoryRequest<'a>,
) -> HistoryCheck<&'a [GeneratedAreaSnapshot]> {
    match &candidate_copy::inspection(request)?.generated_topology {
        GeneratedTopologyObservation::Observed(rows) => Ok(rows),
        GeneratedTopologyObservation::NotReached => Err(PrismHistoryCause::Missing {
            required: "generated topology observation",
        }),
        GeneratedTopologyObservation::Incomplete { areas, cause_index } => {
            Err(PrismHistoryCause::UnsupportedDiagnostic {
                stage: PrismHistoryStage::CandidateCopy,
                kind: HistoricalDiagnosticKind::GeneratedObservation,
                observation: Some(*cause_index),
                retained_rows: areas.len(),
            })
        }
    }
}
pub(super) fn area<'a>(
    request: &PrismHistoryRequest<'a>,
    bound: &BoundHistoricalContexts<'_>,
    stage: PrismHistoryStage,
) -> HistoryCheck<&'a GeneratedAreaSnapshot> {
    let rows = areas(request)?;
    history_count("complete two-region generated topology", rows.len(), 2)?;
    let mut found = Vec::new();
    for row in rows {
        if operands::observed_contexts(&row.contexts, stage)? == bound.actual {
            found.push(row);
        }
    }
    history_count(
        "one actual generated area for semantic candidate",
        found.len(),
        1,
    )?;
    verify_ordinal(request, found[0], stage)?;
    Ok(found[0])
}
fn verify_ordinal(
    request: &PrismHistoryRequest<'_>,
    area: &GeneratedAreaSnapshot,
    stage: PrismHistoryStage,
) -> HistoryCheck<()> {
    let batch = operands::candidates(request)?;
    let actual = batch
        .candidates
        .get(area.ordinal)
        .ok_or(PrismHistoryCause::Count {
            required: "generated ordinal within actual retained batch",
            actual: area.ordinal,
            expected: batch.candidates.len(),
        })?;
    history_count(
        "generated ordinal identifies actual semantic candidate",
        usize::from(
            actual
                .face
                .is_equal(&operands::candidate(request, stage)?.face),
        ),
        1,
    )
}
pub(super) fn project(
    request: &PrismHistoryRequest<'_>,
    bound: &BoundHistoricalContexts<'_>,
    copied: &BoundHistoricalContexts<'_>,
    stage: PrismHistoryStage,
) -> HistoryCheck<Vec<PrismGeneratedMember>> {
    let rows = &area(request, bound, stage)?.members;
    history_count(
        "one generated member row per directed context",
        rows.len(),
        bound.actual.len(),
    )?;
    let mut result = Vec::new();
    for row in rows {
        result.push(member(request, bound, copied, row)?);
    }
    history_count(
        "unique generated directed member rows",
        result
            .iter()
            .map(|r| &r.occurrence)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        bound.actual.len(),
    )?;
    Ok(observed_values::sorted(result))
}
fn member(
    request: &PrismHistoryRequest<'_>,
    bound: &BoundHistoricalContexts<'_>,
    copied: &BoundHistoricalContexts<'_>,
    row: &GeneratedMemberSnapshot,
) -> HistoryCheck<PrismGeneratedMember> {
    let index = parent_rows::occurrence_index(bound, &row.occurrence)?;
    observed_values::equal(
        &(row.kind as u8),
        &bound.owned[index].occurrence.member.kind,
        "generated kind matches actual context",
    )?;
    let descendants = descendants(bound, index, &row.descendants)?;
    let copies = copy_members(request, copied, row)?;
    Ok(PrismGeneratedMember {
        occurrence: bound.owned[index].key.clone(),
        kind: row.kind as u8,
        point_bits: observed_values::point(&row.vertex_point)?,
        descendants,
        copies,
        parents: parent_rows::parents(request, &row.preserved, &row.modified, &row.generated)?,
    })
}
fn descendants(
    bound: &BoundHistoricalContexts<'_>,
    index: usize,
    actual: &[crate::sketch_region::PartitionOccurrenceIdentity],
) -> HistoryCheck<Vec<PrismHistoryUseKey>> {
    let path = &bound.actual[index].child_slots;
    let expected: Vec<_> = bound
        .actual
        .iter()
        .enumerate()
        .filter(|(_, row)| row.child_slots.len() > path.len() && row.child_slots.starts_with(path))
        .collect();
    history_count(
        "complete actual generated descendant census",
        actual.len(),
        expected.len(),
    )?;
    let mut result = Vec::new();
    for identity in actual {
        let matches: Vec<_> = expected
            .iter()
            .filter(|(_, r)| &r.occurrence == identity)
            .collect();
        history_count(
            "one exact generated descendant occurrence",
            matches.len(),
            1,
        )?;
        result.push(bound.owned[matches[0].0].key.clone());
    }
    history_count(
        "unique generated descendant uses",
        result
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        expected.len(),
    )?;
    Ok(observed_values::sorted(result))
}
fn copy_members(
    request: &PrismHistoryRequest<'_>,
    copied: &BoundHistoricalContexts<'_>,
    row: &GeneratedMemberSnapshot,
) -> HistoryCheck<Vec<PrismHistoryMemberKey>> {
    let expected = candidate_copy::tree(request)?
        .iter()
        .filter(|c| same_definition(&c.occurrence, &row.occurrence))
        .map(|c| candidate_copy::tree_key(request, copied, c.index))
        .collect::<HistoryCheck<Vec<_>>>()?;
    let actual = row
        .copy_indices
        .iter()
        .map(|i| candidate_copy::tree_key(request, copied, *i))
        .collect::<HistoryCheck<Vec<_>>>()?;
    observed_values::equal(
        &observed_values::sorted(actual.clone()),
        &observed_values::sorted(expected),
        "generated copy indices exactly bind actual reused physical members",
    )?;
    Ok(observed_values::sorted(actual))
}
