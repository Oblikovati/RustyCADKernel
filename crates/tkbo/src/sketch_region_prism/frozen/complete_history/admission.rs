// SPDX-License-Identifier: GPL-2.0-only
//! Bounded traversal precedes legacy recapture; authored identities and query rows are never truncated.
use super::*;
pub(super) fn admit(request: &PrismHistoryRequest<'_, '_>) -> HistoryCheck<()> {
    history_count(
        "complete history behavior version",
        request.requested.version as usize,
        1,
    )?;
    request
        .requested
        .limits
        .admit()
        .map_err(PrismHistoryCause::Bounds)?;
    history_count("one Jordan source", request.sources.len(), 1)?;
    admit_names(request)?;
    budget::admit(request)?;
    preflight(request)
}
fn preflight(request: &PrismHistoryRequest<'_, '_>) -> HistoryCheck<()> {
    let roots = live_roots(request);
    for root in roots {
        bounds::preflight_history_root(root, request.requested.limits)
            .map_err(PrismHistoryCause::Bounds)?;
    }
    Ok(())
}

fn admit_names(request: &PrismHistoryRequest<'_, '_>) -> HistoryCheck<()> {
    let ids = [
        request.publication.operation,
        request.publication.selected_region,
        request.historical.complementary_region,
        request.joined.closure_contact(),
    ];
    history_count(
        "nonzero distinct operation, selected/complement region and contact identities",
        ids.iter()
            .filter(|i| i.0 != 0)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        ids.len(),
    )?;
    Ok(())
}

fn live_roots<'a>(
    request: &PrismHistoryRequest<'a, '_>,
) -> Vec<&'a rustycad_tkbrep::topods::Shape> {
    let mut roots = vec![request.sources[0].shape];
    if let crate::sketch_region::ArrangementCompletion::Arranged(owned) =
        &request.bridge.mapped.arrangement.completion
    {
        roots.push(&owned.arrangement);
    }
    if let Some(copy) = &request.bridge.mapped.copied
        && let Ok(batch) = &copy.completion
    {
        roots.extend(batch.candidates.iter().map(|c| c.face.as_ref()));
    }
    receipt_roots(request.receipt, &mut roots);
    roots
}

fn receipt_roots<'a>(
    receipt: &'a crate::sketch_region_prism::PrismReceipt,
    roots: &mut Vec<&'a rustycad_tkbrep::topods::Shape>,
) {
    if let Some(selected) = &receipt.input.selected {
        roots.push(selected.face.as_ref());
    }
    if let Some(copied) = &receipt.copied {
        roots.push(&copied.base);
    }
    if let Some(diagnostic) = &receipt.diagnostic {
        roots.push(&diagnostic.body);
    }
}
