// SPDX-License-Identifier: GPL-2.0-only
//! Sweep receipt dispositions bind one-to-one to authored candidates; no rejected receipt is unused.
use super::super::complete_history::{observed_values, parent_rows};
use super::candidates::AnnularCandidateHistory;
use super::sources::bound_scope;
use super::*;
use crate::sketch_region::{ArrangementRelation, PartitionOccurrenceIdentity};

/// Bind the selected and every rejected receipt history to exactly one authored candidate.
pub(super) fn admit(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
    candidates: &[AnnularCandidateHistory],
) -> HistoryCheck<()> {
    selected(request, scopes, candidates)?;
    let rejected =
        |c: &&AnnularCandidateHistory| matches!(c.scope, AnnularHistoryScope::Rejected(_));
    let rejected: Vec<_> = candidates.iter().filter(rejected).collect();
    let receipts = &request.receipt.input.rejected;
    let required = "one rejected receipt history per rejected candidate";
    history_count(required, receipts.len(), rejected.len())?;
    let mut consumed = std::collections::BTreeSet::new();
    for receipt in receipts {
        consumed.insert(rejected_owner(request, scopes, &rejected, receipt)?);
    }
    let required = "rejected receipts bind distinct candidates";
    history_count(required, consumed.len(), rejected.len())?;
    empty_dispositions(request)
}
fn selected(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
    candidates: &[AnnularCandidateHistory],
) -> HistoryCheck<()> {
    let selected =
        |c: &&AnnularCandidateHistory| matches!(c.scope, AnnularHistoryScope::Selected(_));
    let rows: Vec<_> = candidates.iter().filter(selected).collect();
    history_count("one selected authored candidate", rows.len(), 1)?;
    let missing = PrismHistoryCause::Missing {
        required: "selected input history",
    };
    let current = request.receipt.input.selected.as_ref().ok_or(missing)?;
    let binding = super::census::authored_candidate(request, rows[0].name)?;
    let face = &super::scopes::candidate(request, &binding.actual)?.face;
    holds(
        "selected receipt is actual admitted candidate",
        current.face.is_equal(face),
    )?;
    let bound = bound_scope(scopes, rows[0].scope)?;
    let actual = parent_rows::relation_rows(request.publication, bound, &current.relations)?;
    let required = "selected receipt equals actual candidate full relations";
    observed_values::equal(&actual, &rows[0].relations, required)
}
/// The unique rejected candidate whose full relation rows equal this receipt.
fn rejected_owner(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
    rejected: &[&AnnularCandidateHistory],
    receipt: &[ArrangementRelation],
) -> HistoryCheck<u64> {
    let mut owners = Vec::new();
    for candidate in rejected {
        let bound = bound_scope(scopes, candidate.scope)?;
        if !same_outputs(bound, receipt) {
            continue;
        }
        let actual = parent_rows::relation_rows(request.publication, bound, receipt)?;
        let required = "rejected receipt equals actual candidate full relations";
        observed_values::equal(&actual, &candidate.relations, required)?;
        owners.push(candidate.name);
    }
    let required = "one authored candidate owns each rejected receipt";
    history_count(required, owners.len(), 1)?;
    Ok(owners[0])
}
fn same_outputs(bound: &BoundHistoricalContexts<'_>, receipt: &[ArrangementRelation]) -> bool {
    let known = |r: &ArrangementRelation| {
        let output = PartitionOccurrenceIdentity::capture(&r.output);
        bound.actual.iter().any(|c| c.occurrence == output)
    };
    receipt.len() == bound.actual.len() && receipt.iter().all(known)
}
fn empty_dispositions(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> HistoryCheck<()> {
    let batch = super::scopes::batch(request)?;
    let authored = request.region.identities.candidates.len();
    let required = "one batch candidate per authored candidate";
    history_count(required, batch.candidates.len(), authored)?;
    let input = &request.receipt.input;
    for (required, actual) in [
        ("batch original deletion", batch.input_deleted.len()),
        ("batch unrepresented", batch.unrepresented.len()),
        ("receipt original deletion", input.input_deleted.len()),
        ("receipt unrepresented", input.unrepresented.len()),
    ] {
        history_count(required, actual, 0)?;
    }
    Ok(())
}
