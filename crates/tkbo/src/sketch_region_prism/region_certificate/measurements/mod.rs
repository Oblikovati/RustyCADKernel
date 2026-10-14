// SPDX-License-Identifier: GPL-2.0-only
//! ADR0027: source-bound scalar subset, separate from body acceptance and publication.
mod scalars;
mod types;
use super::super::surface_charts::*;
#[cfg(test)]
use super::current_binding as replay;
use super::current_binding::*;
use super::*;
use rustycad_tkmath::predicates::roots::metric_bounds::*;
pub(crate) use types::*;

/// Inspect current cap/sweep measurements; `inspect_prism_measurements(bridge,sources,receipt,options,cancel)`.
/// The owned scalar subset never promotes the mutable diagnostic body to success.
/// External callbacks end before a final fresh source/chart binding pass.
pub(crate) fn inspect_prism_measurements(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    options: PrismMeasureOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<PrismMeasureInspection, PrismMeasureFailure> {
    let captured = capture_current_prism_binding(bridge, sources, receipt, &mut cancelled)?;
    let measures = scalars::measure_subset(
        &captured.region,
        &captured.charts,
        receipt,
        options,
        &mut cancelled,
    )?;
    poll_prism_binding(&mut cancelled)?;
    rebind_current_prism_binding(bridge, sources, receipt, &captured)?;
    Ok(complete_inspection(captured, measures))
}
fn complete_inspection(
    captured: CurrentPrismBinding,
    measures: PrismMeasureSubset,
) -> PrismMeasureInspection {
    PrismMeasureInspection {
        region: captured.region,
        charts: captured.charts,
        measures,
        signed_jacobian: captured.signed_jacobian,
        pending: [PendingMeasure::SideArea, PendingMeasure::TotalArea],
    }
}

#[cfg(test)]
pub(in crate::sketch_region_prism::region_certificate) mod tests;
