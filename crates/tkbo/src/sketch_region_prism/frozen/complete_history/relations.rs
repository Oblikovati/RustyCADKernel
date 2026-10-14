// SPDX-License-Identifier: GPL-2.0-only
//! Copy composition preserves classes; container derivation uses the complete actual descendant census.
use super::*;
use crate::sketch_region::{ArrangementCompletion, OwnedArrangement};
use stages::BoundHistoryStage;
pub(super) fn arranged<'a>(
    request: &PrismHistoryRequest<'a, '_>,
) -> HistoryCheck<&'a OwnedArrangement> {
    match &request.bridge.mapped.arrangement.completion {
        ArrangementCompletion::Arranged(rows) => Ok(rows),
        _ => Err(PrismHistoryCause::UnsupportedDiagnostic {
            stage: PrismHistoryStage::Partition,
            kind: HistoricalDiagnosticKind::ArrangementCompletion,
            observation: None,
            retained_rows: 0,
        }),
    }
}
pub(super) fn partition(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
    queries: &[PrismPartitionQuery],
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    let stage = stages::stage(stages, PrismHistoryStage::Partition)?;
    let owned = arranged(request)?;
    history_count(
        "bounded original partition deletion is empty",
        owned.deleted.len(),
        0,
    )?;
    let rows = parent_rows::relation_rows(request.publication, stage, &owned.relations)?;
    for row in &rows {
        validate_partition(row, stage, queries)?;
    }
    Ok(rows)
}
fn validate_partition(
    row: &PrismHistoricalRelation,
    bound: &BoundHistoricalContexts<'_>,
    queries: &[PrismPartitionQuery],
) -> HistoryCheck<()> {
    let context = relation_context(row, bound)?;
    let is_container =
        context.occurrence.member.kind == rustycad_tkg3d::top_abs::ShapeEnum::Compound as u8;
    let parents = if is_container {
        queries.iter().map(|q| q.original.clone()).collect()
    } else {
        queries
            .iter()
            .filter(|q| q.copied.member.kind == context.occurrence.member.kind)
            .map(|q| q.original.clone())
            .collect()
    };
    let parents = observed_values::sorted(parents);
    let expected = if is_container {
        [vec![], vec![], parents]
    } else {
        [vec![], parents, vec![]]
    };
    same_parents(PrismHistoryStage::Partition, &row.parents, &expected)
}
pub(in crate::sketch_region_prism::frozen) fn same_parents(
    stage: PrismHistoryStage,
    actual: &[Vec<FrozenOriginalKey>; 3],
    expected: &[Vec<FrozenOriginalKey>; 3],
) -> HistoryCheck<()> {
    if actual != expected {
        return Err(PrismHistoryCause::ParentsMismatch {
            stage,
            actual: Box::new(actual.clone()),
            expected: Box::new(expected.clone()),
        });
    }
    Ok(())
}
pub(super) fn generated(
    request: &PrismHistoryRequest<'_, '_>,
    bound: &BoundHistoricalContexts<'_>,
    stage: PrismHistoryStage,
    rows: &[PrismGeneratedMember],
    links: &[PrismCandidateCopyLink],
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    generated_relations(
        request.publication,
        bound,
        stage,
        &operands::candidate(request, stage)?.relations,
        rows,
        links,
    )
}
pub(in crate::sketch_region_prism::frozen) fn generated_relations(
    publication: &PrismPublicationBindings,
    bound: &BoundHistoricalContexts<'_>,
    stage: PrismHistoryStage,
    native: &[crate::sketch_region::ArrangementRelation],
    rows: &[PrismGeneratedMember],
    links: &[PrismCandidateCopyLink],
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    let actual = parent_rows::relation_rows(publication, bound, native)?;
    for row in rows {
        let parents = generated_parents(row, rows, links);
        same_parents(stage, &row.parents, &parents)?;
        let matched: Vec<_> = actual
            .iter()
            .filter(|r| r.output == row.occurrence)
            .collect();
        history_count(
            "one current candidate relation for captured generated row",
            matched.len(),
            1,
        )?;
        same_parents(stage, &matched[0].parents, &row.parents)?;
    }
    Ok(actual)
}
fn generated_parents(
    row: &PrismGeneratedMember,
    rows: &[PrismGeneratedMember],
    links: &[PrismCandidateCopyLink],
) -> [Vec<FrozenOriginalKey>; 3] {
    if !row.copies.is_empty() {
        return copied_parents(&row.copies, links);
    }
    let mut parents = Vec::new();
    for child in rows
        .iter()
        .filter(|r| row.descendants.contains(&r.occurrence))
    {
        parents.extend(copied_parents(&child.copies, links).into_iter().flatten());
    }
    parents.sort();
    parents.dedup();
    [vec![], vec![], parents]
}
fn copied_parents(
    copies: &[PrismHistoryMemberKey],
    links: &[PrismCandidateCopyLink],
) -> [Vec<FrozenOriginalKey>; 3] {
    let mut result: [Vec<FrozenOriginalKey>; 3] = Default::default();
    for row in links.iter().filter(|l| copies.contains(&l.image.member)) {
        result[1].extend(row.parents[0].iter().chain(&row.parents[1]).cloned());
        result[2].extend(row.parents[2].iter().cloned());
    }
    for parents in &mut result {
        parents.sort();
        parents.dedup();
    }
    result
}
pub(super) fn receipt_regions(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
    selected: &[PrismHistoricalRelation],
    complement: &[PrismHistoricalRelation],
) -> HistoryCheck<()> {
    selected_receipt(request, stages, selected)?;
    let input = &request.receipt.input;
    history_count(
        "one actual complementary rejected history",
        input.rejected.len(),
        1,
    )?;
    let actual = parent_rows::relation_rows(
        request.publication,
        stages::stage(stages, PrismHistoryStage::Complement)?,
        &input.rejected[0],
    )?;
    observed_values::equal(
        &actual,
        &complement.to_vec(),
        "rejected receipt equals actual complementary candidate full relations",
    )?;
    empty_input_dispositions(request)
}
fn empty_input_dispositions(request: &PrismHistoryRequest<'_, '_>) -> HistoryCheck<()> {
    let batch = operands::candidates(request)?;
    history_count(
        "exactly selected plus complementary candidate",
        batch.candidates.len(),
        2,
    )?;
    empty_dispositions(request, batch)?;
    Ok(())
}

fn relation_context<'a>(
    row: &PrismHistoricalRelation,
    bound: &'a BoundHistoricalContexts<'_>,
) -> HistoryCheck<&'a PrismHistoricalContext> {
    let context =
        bound
            .owned
            .iter()
            .find(|c| c.key == row.output)
            .ok_or(PrismHistoryCause::Missing {
                required: "partition relation context",
            })?;
    Ok(context)
}

fn selected_receipt(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
    selected: &[PrismHistoricalRelation],
) -> HistoryCheck<()> {
    let input = &request.receipt.input;
    let current = input.selected.as_ref().ok_or(PrismHistoryCause::Missing {
        required: "selected input history",
    })?;
    admit_selected_face(request, &current.face)?;
    let actual = parent_rows::relation_rows(
        request.publication,
        stages::stage(stages, PrismHistoryStage::Selected)?,
        &current.relations,
    )?;
    observed_values::equal(
        &actual,
        &selected.to_vec(),
        "selected receipt equals actual candidate full relations",
    )?;
    Ok(())
}

fn admit_selected_face(
    request: &PrismHistoryRequest<'_, '_>,
    face: &rustycad_tkbrep::topods::Shape,
) -> HistoryCheck<()> {
    history_count(
        "selected receipt is actual admitted candidate",
        usize::from(
            face.is_equal(&operands::candidate(request, PrismHistoryStage::Selected)?.face),
        ),
        1,
    )?;
    Ok(())
}

fn empty_dispositions(
    request: &PrismHistoryRequest<'_, '_>,
    batch: &crate::sketch_region_candidates::CandidateBatch,
) -> HistoryCheck<()> {
    for (required, actual) in [
        ("batch original deletion", batch.input_deleted.len()),
        ("batch unrepresented", batch.unrepresented.len()),
        (
            "receipt original deletion",
            request.receipt.input.input_deleted.len(),
        ),
        (
            "receipt unrepresented",
            request.receipt.input.unrepresented.len(),
        ),
    ] {
        history_count(required, actual, 0)?;
    }
    Ok(())
}
