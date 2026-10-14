// SPDX-License-Identifier: GPL-2.0-only
//! Scope names come from explicit authored rosters; missing, duplicate or unused rows refuse.
use super::super::complete_history::contexts::bind_historical_contexts;
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
use std::collections::BTreeSet;

/// Bind every captured scope to its authored names, then require globally distinct member keys.
pub(super) fn bind_scopes<'a>(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    operands: &'a [AnnularScopeOperands],
) -> HistoryCheck<Vec<AnnularBoundScope<'a>>> {
    let mut result = Vec::new();
    for operand in operands {
        result.push(AnnularBoundScope {
            scope: operand.scope,
            contexts: bind_scope(request, operand)?,
        });
    }
    admit_disjoint_members(&result)?;
    Ok(result)
}
fn bind_scope<'a>(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    operand: &'a AnnularScopeOperands,
) -> HistoryCheck<BoundHistoricalContexts<'a>> {
    let (operation, limits) = (request.publication.operation, request.requested.limits);
    match derived_names(request, operand)? {
        Some(names) => bind_historical_contexts(operation, &names, &operand.rows, limits),
        None => {
            let names = authored_scope(request, operand.scope)?;
            bind_historical_contexts(operation, names, &operand.rows, limits)
        }
    }
}
/// Names derived from publication/copy bindings; `None` selects the explicit authored scope.
fn derived_names(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    operand: &AnnularScopeOperands,
) -> HistoryCheck<Option<PrismHistoricalStageBindings>> {
    let (publication, copied) = (request.publication, request.copied);
    let names = match operand.scope {
        AnnularHistoryScope::Original(source) => original_names(request, source, &operand.rows)?,
        AnnularHistoryScope::Selected(_) => legacy_names(
            operand.scope,
            &copied.selected_members,
            &copied.selected_uses,
        )?,
        AnnularHistoryScope::SweepBase => {
            legacy_names(operand.scope, &publication.sources, &copied.copied_uses)?
        }
        _ => return Ok(None),
    };
    Ok(Some(names))
}
/// The one explicit authored binding for an intermediate scope.
pub(super) fn authored_scope<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    scope: AnnularHistoryScope,
) -> HistoryCheck<&'a PrismHistoricalStageBindings> {
    let scopes = request.historical.scopes.iter();
    let rows: Vec<_> = scopes.filter(|s| s.scope == scope).collect();
    let required = "one authored binding per required intermediate scope";
    history_count(required, rows.len(), 1)?;
    Ok(&rows[0].names)
}
fn original_names(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    source: RegionSourceId,
    rows: &[RegionObservedContext],
) -> HistoryCheck<PrismHistoricalStageBindings> {
    let originals = &request.publication.originals;
    let own = originals.iter().filter(|b| b.actual.source == source);
    let required = "one original authored binding per actual source use";
    history_count(required, own.count(), rows.len())?;
    let mut result = empty(AnnularHistoryScope::Original(source));
    for row in rows {
        let actual = original_identity(source, &row.occurrence);
        let found: Vec<_> = originals.iter().filter(|b| b.actual == actual).collect();
        history_count("unique actual original use binding", found.len(), 1)?;
        append_historical_member(&mut result, &row.occurrence, found[0].member)?;
        let (actual, authored_use) = (row.clone(), found[0].directed_use);
        result.uses.push(PrismHistoricalUseBinding {
            actual,
            authored_use,
        });
    }
    Ok(result)
}
/// Original identity of one actual source use under its authored source.
pub(super) fn original_identity(
    source: RegionSourceId,
    actual: &crate::sketch_region::PartitionOccurrenceIdentity,
) -> OriginalMemberIdentity {
    OriginalMemberIdentity {
        source,
        topology_address: actual.topology_address,
        location: actual.location.clone(),
        orientation: actual.orientation,
    }
}
fn legacy_names(
    scope: AnnularHistoryScope,
    members: &[PrismSourceBinding],
    uses: &[PrismDirectedBinding],
) -> HistoryCheck<PrismHistoricalStageBindings> {
    let mut result = empty(scope);
    for binding in members {
        append_historical_member(&mut result, &binding.actual, binding.member)?;
    }
    for binding in uses {
        result.uses.push(PrismHistoricalUseBinding {
            actual: snapshot_directed_history_context(&binding.actual)?,
            authored_use: binding.authored_use,
        });
    }
    Ok(result)
}
fn empty(scope: AnnularHistoryScope) -> PrismHistoricalStageBindings {
    PrismHistoricalStageBindings {
        stage: scope.stage(),
        members: Vec::new(),
        uses: Vec::new(),
    }
}
fn admit_disjoint_members(scopes: &[AnnularBoundScope<'_>]) -> HistoryCheck<()> {
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for scope in scopes {
        let owned = scope.contexts.owned.iter();
        let members: BTreeSet<_> = owned.map(|c| c.occurrence.member).collect();
        total += members.len();
        seen.extend(members);
    }
    let required = "authored member keys are distinct across every scope";
    history_count(required, seen.len(), total)
}
