// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed native history resolves every ancestor; lookup position never determines a role.
use super::*;
pub(super) fn copied_contexts(
    observed: &RegionCopyContexts,
    source: &[NamedSourceContext],
) -> ResolveCheck<Vec<NamedSourceContext>> {
    complete_context_coverage(source, &observed.source)?;
    require_count(
        "exhaustive native copy query census",
        observed.queries.len(),
        source.len(),
    )?;
    let result = source
        .iter()
        .map(|row| copied_context(observed, row))
        .collect::<ResolveCheck<Vec<_>>>()?;
    complete_context_coverage(&result, &observed.copied)?;
    Ok(result)
}

fn copied_context(
    observed: &RegionCopyContexts,
    row: &NamedSourceContext,
) -> ResolveCheck<NamedSourceContext> {
    let index = resolve_native_copy_context(observed, &row.actual, &observed.copied)?;
    Ok(NamedSourceContext {
        actual: observed.copied[index].clone(),
        member: row.member,
        directed_use: row.directed_use,
    })
}
