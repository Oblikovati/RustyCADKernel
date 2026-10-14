// SPDX-License-Identifier: GPL-2.0-only
//! Original operation settings stay explicit; exact determinant evidence never widens a budget.
use super::super::super::{
    frozen::{FrozenPrismLimits, capture_prism_limits},
    surface_charts::PrismSurfaceCause,
};
use super::*;
use rustycad_tkmath::predicates::roots::{DyadicBound, metric_bounds::affine_prism_jacobian};

pub(super) fn capture(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
) -> AnnularCheck<'static, FrozenPrismLimits> {
    super::super::super::transaction::admit_request(&receipt.limits.requested)
        .map_err(|cause| Box::new(AnnularBindingCause::Body(Box::new(cause))))?;
    holds(
        "retained diagnostic remains incomplete",
        matches!(receipt.completion, PrismCompletion::Incomplete),
    )?;
    let limits = capture_prism_limits(receipt);
    holds(
        "certified actual material selection seed",
        limits.seed.map(f64::to_bits) == request.material_seed.map(f64::to_bits),
    )?;
    holds(
        "effective actual seed",
        limits.effective_seed.map(|v| v.map(f64::to_bits)) == Some(limits.seed.map(f64::to_bits)),
    )?;
    holds(
        "effective actual sweep vector",
        limits.effective_extrusion.map(|v| v.map(f64::to_bits))
            == Some(limits.extrusion.map(f64::to_bits)),
    )?;
    // The diagnostic has not executed tolerance validation. A requested budget is not an applied one.
    holds(
        "diagnostic spatial limit remains unapplied",
        limits.effective_spatial.is_none(),
    )?;
    holds(
        "diagnostic parameter limit remains unapplied",
        limits.effective_parameter.is_none(),
    )?;
    holds(
        "effective roots unchanged",
        limits.effective_roots == Some(limits.roots),
    )?;
    holds(
        "diagnostic does not invent achieved error",
        limits.achieved_error.is_none(),
    )?;
    Ok(limits)
}
pub(super) fn jacobian(
    copy: &AnnularCopyCapture,
    limits: &FrozenPrismLimits,
) -> AnnularCheck<'static, DyadicBound> {
    let support = &copy.copied.support;
    // This is bounded constant-size exact determinant arithmetic; the enclosing stage polls.
    let determinant = affine_prism_jacobian(
        support.frame,
        support.placement,
        limits.extrusion,
        &mut || false,
    )
    .map_err(|cause| Box::<AnnularBindingCause<'static>>::from(PrismSurfaceCause::Exact(cause)))?;
    holds(
        "nonzero exact signed annular sweep Jacobian",
        determinant.compare_f64(0.).is_ok_and(|v| !v.is_eq()),
    )?;
    Ok(determinant)
}
