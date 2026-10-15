// SPDX-License-Identifier: GPL-2.0-only
//! Version, authored rosters and the checked aggregate budget are admitted before any projection.
use super::*;
use std::collections::BTreeSet;

/// Admit one V2 request; returns measured stored operand units. Example: `admit(&request)?`.
pub(super) fn admit(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> HistoryCheck<usize> {
    let version = request.requested.version as usize;
    history_count("complete annular history behavior version", version, 2)?;
    let limits = request.requested.limits;
    limits.admit().map_err(PrismHistoryCause::Bounds)?;
    admit_sources(request)?;
    admit_names(request)?;
    admit_candidates(request)?;
    admit_scopes(request)?;
    super::budget::admit(request)
}
fn admit_sources(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> HistoryCheck<()> {
    let identities = request.region.identities;
    let sources = request.region.sources.len();
    history_count("two authored annular sources", sources, 2)?;
    let distinct = identities.outer != identities.inner;
    holds("distinct outer and inner source roles", distinct)?;
    for source in super::scopes::source_roles(request) {
        super::scopes::source_shape(request, source)?;
    }
    Ok(())
}
fn admit_names(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> HistoryCheck<()> {
    let publication = request.publication;
    let ids = [publication.operation, publication.selected_region];
    let nonzero: BTreeSet<_> = ids.iter().filter(|i| i.0 != 0).collect();
    let required = "nonzero distinct operation and selected region identities";
    history_count(required, nonzero.len(), ids.len())?;
    let same = request.region.identities.operation == publication.operation.0;
    holds("same authored operation for region and publication", same)
}
/// Authored candidate names equal the sealed proof's names; exactly one is the sealed selection.
fn admit_candidates(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> HistoryCheck<()> {
    let identities = request.region.identities;
    let region = request.binding().region();
    let authored: BTreeSet<_> = identities.candidates.iter().map(|c| c.name).collect();
    let nonzero = authored.iter().filter(|n| **n != 0).count();
    let required = "distinct nonzero authored candidates";
    history_count(required, nonzero, identities.candidates.len())?;
    let sealed: BTreeSet<_> = region.candidates().iter().map(|c| c.name).collect();
    holds(
        "authored candidates equal sealed proof candidates",
        authored == sealed,
    )?;
    let selected = region.selected_candidate_id() == identities.selected;
    holds("authored selection is the sealed selection", selected)?;
    let operation = region.operation_id() == identities.operation;
    holds("sealed proof belongs to authored operation", operation)?;
    let named = authored.contains(&identities.selected);
    holds("authored selection names one candidate", named)
}
/// The explicit roster is exactly both InputCopy sources, Partition, CandidateCopy and each Rejected.
fn admit_scopes(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> HistoryCheck<()> {
    let actual: Vec<_> = request.historical.scopes.iter().map(|s| s.scope).collect();
    let distinct: BTreeSet<_> = actual.iter().copied().collect();
    history_count(
        "no duplicate authored history scope",
        distinct.len(),
        actual.len(),
    )?;
    let exact = distinct == expected_scopes(request);
    holds("exact authored intermediate scope roster, no extras", exact)?;
    for scope in &request.historical.scopes {
        let same = scope.names.stage == scope.scope.stage();
        holds("authored scope stage matches scope", same)?;
    }
    Ok(())
}
fn expected_scopes(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
) -> BTreeSet<AnnularHistoryScope> {
    let sources = super::scopes::source_roles(request).into_iter();
    let mut result: BTreeSet<_> = sources.map(AnnularHistoryScope::InputCopy).collect();
    result.insert(AnnularHistoryScope::Partition);
    result.insert(AnnularHistoryScope::CandidateCopy);
    for candidate in &request.region.identities.candidates {
        if candidate.name != request.region.identities.selected {
            result.insert(AnnularHistoryScope::Rejected(candidate.name));
        }
    }
    result
}
