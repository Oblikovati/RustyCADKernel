// SPDX-License-Identifier: GPL-2.0-only
//! ADR0027: source-bound scalar subset, separate from body acceptance and publication.
mod binding;
mod charts;
mod replay;
mod scalars;
mod types;
use super::super::surface_charts::*;
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
    let (region, signed_jacobian) =
        binding::current_region(bridge, sources, receipt, &mut cancelled)?;
    let charts = charts::current_charts(receipt, &mut cancelled)?;
    let measures = scalars::measure_subset(&region, &charts, receipt, options, &mut cancelled)?;
    binding::poll(&mut cancelled)?;
    replay::rebind(bridge, sources, receipt, &region, &charts)?;
    Ok(complete_inspection(
        region,
        charts,
        measures,
        signed_jacobian,
    ))
}
fn complete_inspection(
    region: RegionCertificate,
    charts: Vec<MeasuredPrismChart>,
    measures: PrismMeasureSubset,
    signed_jacobian: rustycad_tkmath::predicates::roots::DyadicBound,
) -> PrismMeasureInspection {
    PrismMeasureInspection {
        region,
        charts,
        measures,
        signed_jacobian,
        pending: [PendingMeasure::SideArea, PendingMeasure::TotalArea],
    }
}

#[cfg(test)]
mod tests;
