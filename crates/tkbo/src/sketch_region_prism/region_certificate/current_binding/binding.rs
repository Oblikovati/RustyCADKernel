// SPDX-License-Identifier: GPL-2.0-only
//! Rerun existing winding and region proofs; mutable receipt status is never evidence.
use super::super::types::{holds, need};
use super::*;

pub(super) fn current_region(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
) -> PrismBindingCheck<(
    RegionCertificate,
    rustycad_tkmath::predicates::roots::DyadicBound,
)> {
    poll(cancelled)?;
    holds(
        "measurement diagnostic remains incomplete",
        matches!(receipt.completion, PrismCompletion::Incomplete),
    )?;
    current_selection(bridge, receipt, cancelled)?;
    let region = inspect::inspect(bridge, sources, receipt)?;
    let jacobian = current_sweep(&region, receipt, cancelled)?;
    poll(cancelled)?;
    Ok((region, jacobian))
}
fn current_selection(
    bridge: &PairedContactTransaction,
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
) -> PrismBindingCheck {
    let batch = need(borrowed_batch(bridge), "measurement actual candidate batch")?;
    let request = receipt.limits.requested;
    let fresh = classify_candidates(batch, request.seed, request.roots, cancelled)
        .map_err(|cause| PrismBindingFailure::Classification(Box::new(cause)))?;
    let old = need(
        receipt.classification.as_ref(),
        "measurement retained classification",
    )?;
    same_classification(&fresh, old)
}
fn same_classification(
    fresh: &CandidateClassification,
    old: &CandidateClassification,
) -> PrismBindingCheck {
    let CandidateClassification::Incomplete {
        selection: first,
        captured: a,
        ..
    } = fresh;
    let CandidateClassification::Incomplete {
        selection: second,
        captured: b,
        ..
    } = old;
    holds(
        "fresh selected component and exact windings",
        first == second && a.candidate_windings == b.candidate_windings,
    )?;
    Ok(())
}
pub(crate) fn poll(cancelled: &mut impl FnMut() -> bool) -> PrismBindingCheck {
    if cancelled() {
        return Err(PrismBindingFailure::Cancelled);
    }
    Ok(())
}

fn current_sweep(
    region: &RegionCertificate,
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
) -> PrismBindingCheck<rustycad_tkmath::predicates::roots::DyadicBound> {
    let support = &region.copied.support;
    let vector = receipt.limits.requested.extrusion;
    let determinant = affine_prism_jacobian(
        support.frame,
        support.placement,
        [vector.x(), vector.y(), vector.z()],
        cancelled,
    )
    .map_err(PrismSurfaceCause::Exact)?;
    let nonzero = determinant
        .compare_f64(0.)
        .is_ok_and(|order| !order.is_eq());
    holds("fresh nonzero measurement sweep determinant", nonzero)?;
    Ok(determinant)
}
