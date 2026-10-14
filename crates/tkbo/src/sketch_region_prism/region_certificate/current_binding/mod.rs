// SPDX-License-Identifier: GPL-2.0-only
//! One fresh source/chart binding path shared by private inspections.
mod binding;
mod charts;
#[allow(unused_imports)]
pub(in crate::sketch_region_prism) use charts::capture_actual_charts;
mod replay;
mod types;
use super::super::surface_charts::*;
use super::*;
#[allow(unused_imports)]
pub(in crate::sketch_region_prism) use binding::current_selection as rebind_prism_selection;
pub(crate) use binding::poll as poll_prism_binding;
#[cfg(test)]
pub(crate) use replay::same_charts;
use rustycad_tkmath::predicates::roots::metric_bounds::*;
pub(crate) use types::*;

/// Capture current region and actual trimmed charts; `capture_current_prism_binding(bridge,sources,receipt,cancel)`.
/// This owns coefficients and operation-local identities, never a published body.
pub(crate) fn capture_current_prism_binding(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<CurrentPrismBinding, PrismBindingFailure> {
    let (region, signed_jacobian) = binding::current_region(bridge, sources, receipt, cancelled)?;
    let charts = charts::current_charts(receipt, cancelled)?;
    Ok(CurrentPrismBinding {
        region,
        signed_jacobian,
        charts,
    })
}

/// Recheck after all callbacks; `rebind_current_prism_binding(bridge,sources,receipt,&captured)`.
/// No external callback can mutate inputs during this final comparison.
pub(crate) fn rebind_current_prism_binding(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    captured: &CurrentPrismBinding,
) -> Result<(), PrismBindingFailure> {
    replay::rebind(bridge, sources, receipt, &captured.region, &captured.charts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_current_prism_binding_owns_send_sync_operands() {
        fn check<T: Send + Sync>() {}
        check::<CurrentPrismBinding>();
        check::<BoundPrismChart>();
    }
}
