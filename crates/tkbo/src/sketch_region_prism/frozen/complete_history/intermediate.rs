// SPDX-License-Identifier: GPL-2.0-only
//! One historical capture feeds both initial proof and final callback-free comparison.
use super::*;
use stages::BoundHistoryStage;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrismIntermediateHistory {
    pub contexts: Vec<PrismHistoricalContext>,
    pub initial_copy: Vec<PrismHistoricalCopyQuery>,
    pub partition_queries: Vec<PrismPartitionQuery>,
    pub partition_relations: Vec<PrismHistoricalRelation>,
    pub candidate_copy: Vec<PrismHistoricalCopyQuery>,
    pub candidates: PrismCandidateHistory,
    pub selected_relations: Vec<PrismHistoricalRelation>,
    pub complementary_relations: Vec<PrismHistoricalRelation>,
}
pub(super) fn capture(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
) -> HistoryCheck<PrismIntermediateHistory> {
    let initial_copy = initial_queries(request, stages)?;
    let partition_queries = partition_queries::project_partition_queries(request, stages)?;
    let partition_relations = relations::partition(request, stages, &partition_queries)?;
    let candidate_copy = candidate_queries(request, stages)?;
    let candidates = candidate_history(request, stages, &candidate_copy, &partition_relations)?;
    let (selected_relations, complementary_relations) =
        candidate_relations(request, stages, &candidates)?;
    let contexts = all_contexts(stages);
    Ok(PrismIntermediateHistory {
        contexts,
        initial_copy,
        partition_queries,
        partition_relations,
        candidate_copy,
        candidates,
        selected_relations,
        complementary_relations,
    })
}
fn initial_queries(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
) -> HistoryCheck<Vec<PrismHistoricalCopyQuery>> {
    let initial = operands::copied_contexts(
        &operands::partition(request)?.input_contexts[0].observation,
        PrismHistoryStage::InputCopy,
    )?;
    copied_contexts::project_copier_queries(
        initial,
        stages::stage(stages, PrismHistoryStage::Original)?,
        stages::stage(stages, PrismHistoryStage::InputCopy)?,
    )
}
fn candidate_queries(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
) -> HistoryCheck<Vec<PrismHistoricalCopyQuery>> {
    copied_contexts::project_copier_queries(
        stages::candidate_copy(request)?,
        stages::stage(stages, PrismHistoryStage::Partition)?,
        stages::stage(stages, PrismHistoryStage::CandidateCopy)?,
    )
}
fn candidate_history(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
    queries: &[PrismHistoricalCopyQuery],
    relations: &[PrismHistoricalRelation],
) -> HistoryCheck<PrismCandidateHistory> {
    let copied = stages::stage(stages, PrismHistoryStage::CandidateCopy)?;
    let selected = stages::stage(stages, PrismHistoryStage::Selected)?;
    let complement = stages::stage(stages, PrismHistoryStage::Complement)?;
    let copy_links = candidate_copy::copy_links(request, stages, queries, relations)?;
    let copy_tree = candidate_copy::project_tree(&request.native_inputs(), copied)?;
    let directed = generated_directed::project(copied, selected, complement)?;
    let omitted_administrative_container = administrative_container(copied)?;
    Ok(PrismCandidateHistory {
        copy_links,
        copy_tree,
        directed,
        omitted_administrative_container,
        selected: generated::project(request, selected, copied, PrismHistoryStage::Selected)?,
        complement: generated::project(request, complement, copied, PrismHistoryStage::Complement)?,
    })
}
fn administrative_container(
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<PrismHistoryUseKey> {
    let rows: Vec<_> = copied
        .owned
        .iter()
        .filter(|r| r.occurrence.member.kind == rustycad_tkg3d::top_abs::ShapeEnum::Compound as u8)
        .collect();
    history_count(
        "one copied administrative container outside the generated candidate boundary",
        rows.len(),
        1,
    )?;
    Ok(rows[0].key.clone())
}

fn candidate_relations(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
    candidates: &PrismCandidateHistory,
) -> HistoryCheck<(Vec<PrismHistoricalRelation>, Vec<PrismHistoricalRelation>)> {
    let selected_relations =
        generated_relations(request, stages, candidates, PrismHistoryStage::Selected)?;
    let complementary_relations =
        generated_relations(request, stages, candidates, PrismHistoryStage::Complement)?;
    relations::receipt_regions(
        request,
        stages,
        &selected_relations,
        &complementary_relations,
    )?;
    Ok((selected_relations, complementary_relations))
}

fn generated_relations(
    request: &PrismHistoryRequest<'_, '_>,
    stages: &[BoundHistoryStage<'_>],
    candidates: &PrismCandidateHistory,
    stage: PrismHistoryStage,
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    let rows = if stage == PrismHistoryStage::Selected {
        &candidates.selected
    } else {
        &candidates.complement
    };
    relations::generated(
        request,
        stages::stage(stages, stage)?,
        stage,
        rows,
        &candidates.copy_links,
    )
}

fn all_contexts(stages: &[BoundHistoryStage<'_>]) -> Vec<PrismHistoricalContext> {
    observed_values::sorted(
        stages
            .iter()
            .flat_map(|s| s.contexts.owned.clone())
            .collect(),
    )
}
