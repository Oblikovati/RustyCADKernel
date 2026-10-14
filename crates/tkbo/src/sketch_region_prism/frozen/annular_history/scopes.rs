// SPDX-License-Identifier: GPL-2.0-only
//! Actual scope censuses are captured from authored roles: source IDs and candidate names select rows.
use super::*;
use crate::sketch_region::{ArrangementCompletion, RegionCopyContexts};

/// Capture every scope's current or retained rows in authored role order.
pub(super) fn capture_operands(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
) -> HistoryCheck<Vec<AnnularScopeOperands>> {
    let mut result = source_operands(request)?;
    let copied = candidate_copy(request)?.copied.clone();
    result.push(scope(
        AnnularHistoryScope::Partition,
        partition_rows(request)?,
    ));
    result.push(scope(AnnularHistoryScope::CandidateCopy, copied));
    for candidate in &request.region.identities.candidates {
        let rows = candidate_rows(request, &candidate.actual)?;
        result.push(scope(candidate_scope(request, candidate.name), rows));
    }
    result.push(scope(AnnularHistoryScope::SweepBase, sweep_rows(request)?));
    for row in &result {
        census::admit_scope(request, row)?;
    }
    Ok(result)
}
fn source_operands(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
) -> HistoryCheck<Vec<AnnularScopeOperands>> {
    let mut result = Vec::new();
    for source in source_roles(request) {
        let initial = initial_contexts(request, source)?;
        let original = original_rows(request, source, initial)?;
        result.push(scope(AnnularHistoryScope::Original(source), original));
        result.push(scope(
            AnnularHistoryScope::InputCopy(source),
            initial.copied.clone(),
        ));
    }
    Ok(result)
}
fn scope(scope: AnnularHistoryScope, rows: Vec<RegionObservedContext>) -> AnnularScopeOperands {
    AnnularScopeOperands { scope, rows }
}
/// Authored outer/inner roles, never request-row order.
pub(super) fn source_roles(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> [RegionSourceId; 2] {
    [
        request.region.identities.outer,
        request.region.identities.inner,
    ]
}
pub(super) fn candidate_scope(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    name: u64,
) -> AnnularHistoryScope {
    if name == request.region.identities.selected {
        AnnularHistoryScope::Selected(name)
    } else {
        AnnularHistoryScope::Rejected(name)
    }
}
/// Resolve the retained input-copy receipt by its authored source identity.
pub(super) fn initial_contexts<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    source: RegionSourceId,
) -> HistoryCheck<&'a RegionCopyContexts> {
    let partition = partition_snapshot(request)?;
    let rows: Vec<_> = partition
        .input_contexts
        .iter()
        .filter(|c| c.source == source)
        .collect();
    history_count(
        "one initial copied context receipt per authored source",
        rows.len(),
        1,
    )?;
    retained_copy_contexts(&rows[0].observation, PrismHistoryStage::InputCopy)
}
pub(super) fn partition_snapshot<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
) -> HistoryCheck<&'a crate::sketch_region::OwnedPartitionSnapshot> {
    super::super::complete_history::operands::partition_native(&request.native_inputs())
}
fn original_rows(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    source: RegionSourceId,
    initial: &RegionCopyContexts,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    let shape = source_shape(request, source)?;
    let rows =
        capture_history_contexts(shape, PrismHistoryStage::Original, request.requested.limits)?;
    history_count(
        "current source exactly matches its historical original context census",
        usize::from(rows == initial.source),
        1,
    )?;
    Ok(rows)
}
/// Borrow the current authored source shape by identity.
pub(super) fn source_shape<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    source: RegionSourceId,
) -> HistoryCheck<&'a rustycad_tkbrep::topods::Shape> {
    let rows: Vec<_> = request
        .region
        .sources
        .iter()
        .filter(|s| s.identity == source)
        .collect();
    history_count("one current request row per authored source", rows.len(), 1)?;
    Ok(rows[0].shape)
}
fn partition_rows(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    let ArrangementCompletion::Arranged(owned) =
        &request.region.bridge.mapped.arrangement.completion
    else {
        return Err(PrismHistoryCause::Missing {
            required: "actual arranged partition root",
        });
    };
    let rows = capture_history_contexts(
        &owned.arrangement,
        PrismHistoryStage::Partition,
        request.requested.limits,
    )?;
    history_count(
        "current partition equals copier source context census",
        usize::from(rows == candidate_copy(request)?.source),
        1,
    )?;
    Ok(rows)
}
fn sweep_rows(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    let copied = request
        .receipt
        .copied
        .as_ref()
        .ok_or(PrismHistoryCause::Missing {
            required: "actual sweep copied base",
        })?;
    capture_history_contexts(
        &copied.base,
        PrismHistoryStage::SweepBase,
        request.requested.limits,
    )
}
pub(super) fn candidate_copy<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
) -> HistoryCheck<&'a RegionCopyContexts> {
    let copied =
        super::super::complete_history::candidate_copy::inspection(&request.native_inputs())?;
    retained_copy_contexts(&copied.contexts, PrismHistoryStage::CandidateCopy)
}
/// Resolve one authored candidate face in the actual completed batch.
pub(super) fn candidate<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    actual: &crate::sketch_region::PartitionOccurrenceIdentity,
) -> HistoryCheck<&'a crate::sketch_region_candidates::RegionCandidate> {
    resolve_certified_candidate(batch(request)?, actual)
}
pub(super) fn batch<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
) -> HistoryCheck<&'a crate::sketch_region_candidates::CandidateBatch> {
    super::super::complete_history::candidate_copy::inspection(&request.native_inputs())?
        .completion
        .as_ref()
        .map_err(|_| PrismHistoryCause::Missing {
            required: "actual completed candidate batch",
        })
}
fn candidate_rows(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    actual: &crate::sketch_region::PartitionOccurrenceIdentity,
) -> HistoryCheck<Vec<RegionObservedContext>> {
    let face = &candidate(request, actual)?.face;
    capture_history_contexts(
        face,
        PrismHistoryStage::Complement,
        request.requested.limits,
    )
}
