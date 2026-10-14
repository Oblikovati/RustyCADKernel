// SPDX-License-Identifier: GPL-2.0-only
//! Reuse exact retained copy images at every ancestor, then require a complete directed bijection.
use super::*;
use crate::sketch_region_prism::frozen::lineage::matches_selected_copy_context;
use crate::sketch_region_prism::region_certificate::current_binding::CurrentPrismBinding;
use crate::sketch_region_prism::{PrismBodyContext, PrismCopyObservation};
pub(super) fn sweep_base_contexts(
    prepared: &PreparedPolynomialPrism,
    binding: &CurrentPrismBinding,
    selected: &[NamedSourceContext],
) -> ResolveCheck<Vec<NamedSourceContext>> {
    let face = generated::certified_face(binding, PrismHistoryStage::Selected)?;
    let candidate = resolve_certified_candidate(current::candidates(prepared)?, face)?;
    let source = current::live_contexts(prepared, &candidate.face, PrismHistoryStage::Selected)?;
    let copy = require_local(
        current::receipt(prepared)?.copied.as_ref(),
        "actual selected copy",
    )?;
    let target = current::live_contexts(prepared, &copy.base, PrismHistoryStage::SweepBase)?;
    map_selected_copy(selected, &source, &target, &copy.observations)
}
pub(super) fn map_selected_copy(
    named: &[NamedSourceContext],
    source: &[PrismBodyContext],
    target: &[PrismBodyContext],
    observations: &[PrismCopyObservation],
) -> ResolveCheck<Vec<NamedSourceContext>> {
    complete_context_coverage(named, &current::snapshot_contexts(source)?)?;
    let mut result = Vec::new();
    for row in named {
        let live = unique_live(source, &row.actual)?;
        let actual = copied_context(observations, live, target)?;
        result.push(NamedSourceContext {
            actual,
            member: row.member,
            directed_use: row.directed_use,
        });
    }
    complete_context_coverage(&result, &current::snapshot_contexts(target)?)?;
    Ok(result)
}
fn unique_live<'a>(
    contexts: &'a [PrismBodyContext],
    expected: &RegionObservedContext,
) -> ResolveCheck<&'a PrismBodyContext> {
    let mut found = Vec::new();
    for context in contexts {
        if &snapshot_directed_history_context(context)? == expected {
            found.push(context);
        }
    }
    require_count(
        "one live selected context for exact named snapshot",
        found.len(),
        1,
    )?;
    Ok(found[0])
}

fn copied_context(
    observations: &[PrismCopyObservation],
    live: &PrismBodyContext,
    target: &[PrismBodyContext],
) -> ResolveCheck<RegionObservedContext> {
    let found: Vec<_> = target
        .iter()
        .filter(|other| matches_selected_copy_context(observations, live, other))
        .collect();
    require_count(
        "unique actual selected-copy directed continuation",
        found.len(),
        1,
    )?;
    Ok(snapshot_directed_history_context(found[0])?)
}
