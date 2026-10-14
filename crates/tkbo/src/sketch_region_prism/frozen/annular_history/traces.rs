// SPDX-License-Identifier: GPL-2.0-only
//! Each copied boundary use continues into the selected annulus and exactly one rejected domain.
use super::super::complete_history::observed_values;
use super::candidates::AnnularCandidateHistory;
use super::sources::bound_scope;
use super::*;
use std::collections::BTreeSet;

/// One copied directed use and its exact generated continuations; wire names are authored roles.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnnularGeneratedTrace {
    pub copied: PrismHistoryUseKey,
    pub selected: PrismHistoryUseKey,
    pub selected_wire: u64,
    pub rejected_candidate: u64,
    pub rejected: PrismHistoryUseKey,
    pub rejected_wire: u64,
}
struct TraceTarget {
    candidate: u64,
    key: PrismHistoryUseKey,
    wire: u64,
}
/// Project every non-container candidate-copy use; both target sides must be exhaustive bijections.
pub(super) fn capture(
    scopes: &[AnnularBoundScope<'_>],
    candidates: &[AnnularCandidateHistory],
) -> HistoryCheck<Vec<AnnularGeneratedTrace>> {
    let copied = bound_scope(scopes, AnnularHistoryScope::CandidateCopy)?;
    let rows = copied.actual.iter().zip(&copied.owned);
    let mut result = Vec::new();
    for (source, owned) in rows.filter(|(r, _)| !r.child_slots.is_empty()) {
        let selected = unique_target(scopes, candidates, source, true)?;
        let rejected = unique_target(scopes, candidates, source, false)?;
        result.push(AnnularGeneratedTrace {
            copied: owned.key.clone(),
            selected: selected.key,
            selected_wire: selected.wire,
            rejected_candidate: rejected.candidate,
            rejected: rejected.key,
            rejected_wire: rejected.wire,
        });
    }
    admit_coverage(scopes, candidates, &result)?;
    Ok(observed_values::sorted(result))
}
fn unique_target(
    scopes: &[AnnularBoundScope<'_>],
    candidates: &[AnnularCandidateHistory],
    source: &RegionObservedContext,
    selected: bool,
) -> HistoryCheck<TraceTarget> {
    let mut found = Vec::new();
    for candidate in candidates.iter().filter(|c| is_selected(c) == selected) {
        let bound = bound_scope(scopes, candidate.scope)?;
        for index in generated_context_matches(source, bound.actual)? {
            let key = bound.owned[index].key.clone();
            let (wire, candidate) = (wire_name(candidate, &key)?, candidate.name);
            found.push(TraceTarget {
                candidate,
                key,
                wire,
            });
        }
    }
    let required = if selected {
        "one selected generated continuation per copied use"
    } else {
        "one rejected generated continuation per copied use"
    };
    history_count(required, found.len(), 1)?;
    Ok(found.remove(0))
}
fn is_selected(candidate: &AnnularCandidateHistory) -> bool {
    matches!(candidate.scope, AnnularHistoryScope::Selected(_))
}
/// The authored wire whose full directed key prefixes this generated use.
fn wire_name(candidate: &AnnularCandidateHistory, key: &PrismHistoryUseKey) -> HistoryCheck<u64> {
    let below = |w: &&super::candidates::AnnularCandidateWire| {
        key.0.len() > w.wire.0.len() && key.0.starts_with(&w.wire.0)
    };
    let rows: Vec<_> = candidate.wires.iter().filter(below).collect();
    let required = "one authored wire ancestor per generated boundary use";
    history_count(required, rows.len(), 1)?;
    Ok(rows[0].name)
}
/// Every generated edge/vertex use of each side is consumed by exactly one trace.
fn admit_coverage(
    scopes: &[AnnularBoundScope<'_>],
    candidates: &[AnnularCandidateHistory],
    rows: &[AnnularGeneratedTrace],
) -> HistoryCheck<()> {
    for selected in [true, false] {
        let expected = boundary_uses(scopes, candidates, selected)?;
        let target = |r: &AnnularGeneratedTrace| {
            if selected {
                (0, r.selected.clone())
            } else {
                (r.rejected_candidate, r.rejected.clone())
            }
        };
        let consumed: BTreeSet<_> = rows.iter().map(target).collect();
        let required = "exhaustive generated boundary-use bijection";
        history_count(required, consumed.len(), expected)?;
        history_count("one trace per generated boundary use", rows.len(), expected)?;
    }
    Ok(())
}
fn boundary_uses(
    scopes: &[AnnularBoundScope<'_>],
    candidates: &[AnnularCandidateHistory],
    selected: bool,
) -> HistoryCheck<usize> {
    let mut result = 0;
    for candidate in candidates.iter().filter(|c| is_selected(c) == selected) {
        let actual = bound_scope(scopes, candidate.scope)?.actual.iter();
        result += actual.filter(|r| r.child_slots.len() >= 2).count();
    }
    Ok(result)
}
