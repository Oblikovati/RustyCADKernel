// SPDX-License-Identifier: GPL-2.0-only
//! Member observations and per-context Modified queries must agree before generation can be composed.
use super::*;
use crate::sketch_region_candidates::{
    CandidateCopiedMember, CandidateCopyInspection, CandidateCopyTreeObservation,
};
use stages::BoundHistoryStage;
pub(super) fn inspection<'a>(
    request: &PrismHistoryRequest<'a>,
) -> HistoryCheck<&'a CandidateCopyInspection> {
    request
        .bridge
        .mapped
        .copied
        .as_ref()
        .ok_or(PrismHistoryCause::Missing {
            required: "candidate copy inspection",
        })
}
pub(super) fn copy_links(
    request: &PrismHistoryRequest<'_>,
    stages: &[BoundHistoryStage<'_>],
    queries: &[PrismHistoricalCopyQuery],
    relations: &[PrismHistoricalRelation],
) -> HistoryCheck<Vec<PrismCandidateCopyLink>> {
    let source = stages::stage(stages, PrismHistoryStage::Partition)?;
    let copied = stages::stage(stages, PrismHistoryStage::CandidateCopy)?;
    let observed = &inspection(request)?.links;
    history_count(
        "exhaustive candidate copy member links",
        observed.len(),
        source.actual.len(),
    )?;
    let mut result = Vec::new();
    for link in observed {
        result.push(copy_link(
            request, source, copied, link, queries, relations,
        )?);
    }
    history_count(
        "unique original context per candidate copy link",
        result
            .iter()
            .map(|r| &r.original)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        source.actual.len(),
    )?;
    Ok(observed_values::sorted(result))
}
fn copy_link(
    request: &PrismHistoryRequest<'_>,
    source: &BoundHistoricalContexts<'_>,
    copied: &BoundHistoricalContexts<'_>,
    link: &crate::sketch_region_candidates::CandidateImageCopyLink,
    queries: &[PrismHistoricalCopyQuery],
    relations: &[PrismHistoricalRelation],
) -> HistoryCheck<PrismCandidateCopyLink> {
    let index = parent_rows::occurrence_index(source, &link.original)?;
    let original = source.owned[index].key.clone();
    let image = observed_values::occurrence(copied, &link.copied)?;
    let parents = parent_rows::parents(request, &link.preserved, &link.modified, &link.generated)?;
    let kind = *partition_queries::native_result(&link.kind)? as u8;
    let supported = *partition_queries::native_result(&link.history_supported)?;
    observed_values::equal(&supported, &(kind != rustycad_tkg3d::top_abs::ShapeEnum::Compound as u8), "supported leaf history versus administrative container")?;
    observed_values::equal(
        &kind,
        &source.owned[index].occurrence.member.kind,
        "copier member kind matches original",
    )?;
    match_link(&original, &image, &parents, queries, relations)?;
    Ok(PrismCandidateCopyLink {
        original,
        image,
        kind,
        supported,
        parents,
    })
}
fn match_link(
    key: &PrismHistoryUseKey,
    image: &PrismHistoricalOccurrence,
    parents: &[Vec<FrozenOriginalKey>; 3],
    queries: &[PrismHistoricalCopyQuery],
    relations: &[PrismHistoricalRelation],
) -> HistoryCheck<()> {
    let queries: Vec<_> = queries.iter().filter(|q| &q.source == key).collect();
    history_count(
        "one retained context query per member link",
        queries.len(),
        1,
    )?;
    observed_values::equal(
        &queries[0].native_images,
        &vec![image.clone()],
        "member link equals actual context Modified answer",
    )?;
    let rows: Vec<_> = relations.iter().filter(|r| &r.output == key).collect();
    history_count("one partition relation per copy link", rows.len(), 1)?;
    observed_values::equal(
        parents,
        &rows[0].parents,
        "copier retains every partition parent and class",
    )
}
pub(super) fn tree<'a>(
    request: &PrismHistoryRequest<'a>,
) -> HistoryCheck<&'a [CandidateCopiedMember]> {
    match &inspection(request)?.copied_tree {
        CandidateCopyTreeObservation::Captured(rows) => Ok(rows),
        CandidateCopyTreeObservation::NotReached => Err(PrismHistoryCause::Missing {
            required: "actual copied physical tree",
        }),
        CandidateCopyTreeObservation::Incomplete { cause, .. } => {
            Err(PrismHistoryCause::Native(cause.clone()))
        }
    }
}
pub(super) fn project_tree(
    request: &PrismHistoryRequest<'_>,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<Vec<PrismCopiedTreeMember>> {
    let rows = tree(request)?;
    let expected: std::collections::BTreeSet<_> =
        copied.owned.iter().map(|r| r.occurrence.member).collect();
    history_count(
        "complete physical candidate-copy tree",
        rows.len(),
        expected.len(),
    )?;
    history_count(
        "unique temporary tree lookup indices",
        rows.iter()
            .map(|r| r.index)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        rows.len(),
    )?;
    let mut result = Vec::new();
    for row in rows {
        result.push(tree_member(copied, row)?);
    }
    observed_values::equal(
        &result.iter().map(|r| r.occurrence.member).collect(),
        &expected,
        "copied tree authored member bijection",
    )?;
    Ok(observed_values::sorted(result))
}
fn tree_member(
    copied: &BoundHistoricalContexts<'_>,
    row: &CandidateCopiedMember,
) -> HistoryCheck<PrismCopiedTreeMember> {
    let occurrence = observed_values::occurrence(copied, &row.occurrence)?;
    observed_values::equal(
        &(row.kind as u8),
        &occurrence.member.kind,
        "copied tree kind matches actual context",
    )?;
    Ok(PrismCopiedTreeMember {
        occurrence,
        point_bits: observed_values::point(&row.vertex_point)?,
    })
}
pub(super) fn tree_key(
    request: &PrismHistoryRequest<'_>,
    copied: &BoundHistoricalContexts<'_>,
    index: usize,
) -> HistoryCheck<PrismHistoryMemberKey> {
    let rows: Vec<_> = tree(request)?.iter().filter(|r| r.index == index).collect();
    history_count("one actual copied tree lookup index", rows.len(), 1)?;
    Ok(observed_values::occurrence(copied, &rows[0].occurrence)?.member)
}
