// SPDX-License-Identifier: GPL-2.0-only
//! Fresh shared region/chart proof owns role authority; retained status flags never do.
use super::*;
use crate::sketch_region_candidates::CandidateBatch;
use crate::sketch_region_prism::region_certificate::current_binding::*;
use crate::sketch_region_prism::{PrismBodyContext, PrismReceipt};
use rustycad_tkbrep::topods::Shape;
pub(super) fn binding(
    prepared: &PreparedPolynomialPrism,
    cancelled: &mut impl FnMut() -> bool,
) -> ResolveCheck<CurrentPrismBinding> {
    let bridge = require_local(
        prepared.retained.bridge.as_ref(),
        "actual contact transaction",
    )?;
    let source = require_local(prepared.retained.source.as_ref(), "constructed source")?;
    capture_current_prism_binding(bridge, &[source.source()], receipt(prepared)?, cancelled)
        .map_err(|cause| ResolutionCause::Binding(Box::new(cause)))
}
pub(super) fn receipt(prepared: &PreparedPolynomialPrism) -> ResolveCheck<&PrismReceipt> {
    require_local(prepared.retained.prism.as_ref(), "actual retained prism")
}
pub(super) fn candidates(prepared: &PreparedPolynomialPrism) -> ResolveCheck<&CandidateBatch> {
    let bridge = require_local(
        prepared.retained.bridge.as_ref(),
        "actual contact transaction",
    )?;
    require_local(
        bridge
            .mapped
            .copied
            .as_ref()
            .and_then(|copy| copy.completion.as_ref().ok()),
        "actual completed candidate batch",
    )
}
pub(super) fn live_contexts(
    prepared: &PreparedPolynomialPrism,
    root: &Shape,
    stage: PrismHistoryStage,
) -> ResolveCheck<Vec<PrismBodyContext>> {
    let limits = prepared.retained.request.policy.publication.history.limits;
    let observed = capture_history_contexts(root, stage, limits)?;
    admit_history_stage_contexts(stage, &observed, limits)?;
    let live = crate::sketch_region_prism::contexts::body_contexts(root)
        .map_err(|cause| ResolutionCause::Prism(Box::new(cause)))?;
    let snapshots = snapshot_contexts(&live)?;
    require_count(
        "live contexts equal fresh bounded census",
        usize::from(snapshots == observed),
        1,
    )?;
    Ok(live)
}

pub(super) fn snapshot_contexts(
    live: &[PrismBodyContext],
) -> ResolveCheck<Vec<RegionObservedContext>> {
    live.iter()
        .map(|row| snapshot_directed_history_context(row).map_err(ResolutionCause::from))
        .collect()
}
