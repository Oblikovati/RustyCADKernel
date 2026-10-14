// SPDX-License-Identifier: GPL-2.0-only
//! Charge authored scopes, native operands and current live-root censuses before any projection.
use super::super::complete_history::budget::{HistoryRowBudget, admit_native};
use super::*;
use rustycad_tkbrep::topods::Shape;

/// Checked aggregate admission; excess refuses without truncation. Example: `admit(&request)?`.
pub(super) fn admit(request: &AnnularHistoryRequest<'_, '_, '_, '_>) -> HistoryCheck<usize> {
    let mut budget = HistoryRowBudget {
        used: 0,
        maximum: request.requested.limits.maximum_rows,
    };
    authored(request, &mut budget)?;
    scopes(request, &mut budget)?;
    admit_native(&request.native_inputs(), &mut budget)?;
    for root in live_roots(request)? {
        let census = preflight_history_census(root, request.requested.limits)
            .map_err(PrismHistoryCause::Bounds)?;
        budget.add(census.directed_uses)?;
        budget.add(census.ancestor_elements)?;
    }
    Ok(budget.used)
}
fn authored(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    budget: &mut HistoryRowBudget,
) -> HistoryCheck<()> {
    let (publication, copied) = (request.publication, request.copied);
    for count in [
        request.region.sources.len(),
        publication.originals.len(),
        publication.sources.len(),
        copied.selected_members.len(),
        request.historical.scopes.len(),
    ] {
        budget.add(count)?;
    }
    for candidate in &request.region.identities.candidates {
        budget.add(1 + candidate.wires.len())?;
    }
    let uses = [
        &publication.directed,
        &copied.selected_uses,
        &copied.copied_uses,
    ];
    uses.into_iter().try_for_each(|rows| directed(rows, budget))
}
fn directed(rows: &[PrismDirectedBinding], budget: &mut HistoryRowBudget) -> HistoryCheck<()> {
    budget.add(rows.len())?;
    for row in rows {
        budget.add(row.actual.ancestors.len())?;
        budget.add(row.actual.child_slots.len())?;
    }
    Ok(())
}
fn scopes(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    budget: &mut HistoryRowBudget,
) -> HistoryCheck<()> {
    for scope in &request.historical.scopes {
        budget.add(scope.names.members.len())?;
        budget.add(scope.names.uses.len())?;
        for row in &scope.names.uses {
            budget.contexts(std::slice::from_ref(&row.actual))?;
        }
    }
    Ok(())
}
/// Every live root later observed by capture; bounded before any context recapture.
fn live_roots<'a>(request: &AnnularHistoryRequest<'a, '_, '_, '_>) -> HistoryCheck<Vec<&'a Shape>> {
    let mut roots = Vec::new();
    for source in super::scopes::source_roles(request) {
        roots.push(super::scopes::source_shape(request, source)?);
    }
    if let crate::sketch_region::ArrangementCompletion::Arranged(owned) =
        &request.region.bridge.mapped.arrangement.completion
    {
        roots.push(&owned.arrangement);
    }
    if let Ok(batch) = super::scopes::batch(request) {
        roots.extend(batch.candidates.iter().map(|c| c.face.as_ref()));
    }
    let receipt = request.receipt;
    roots.extend(receipt.input.selected.as_ref().map(|s| s.face.as_ref()));
    roots.extend(receipt.copied.as_ref().map(|c| &c.base));
    roots.extend(receipt.diagnostic.as_ref().map(|d| &d.body));
    Ok(roots)
}
