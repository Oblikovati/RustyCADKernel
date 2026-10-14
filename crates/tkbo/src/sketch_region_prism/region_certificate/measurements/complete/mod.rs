// SPDX-License-Identifier: GPL-2.0-only
//! ADR0031: all chart measures, conditional on the same current binding as subset v1.
mod operand_errors;
pub(crate) use operand_errors::*;
mod side;
mod types;
use super::*;
pub(crate) use types::*;

/// Inspect cap, side, total and signed-sweep measures; `inspect_complete_prism_measurements(bridge,sources,receipt,options,cancel)`.
/// Completion covers scalar evidence only; this never publishes or validates a body.
pub(crate) fn inspect_complete_prism_measurements(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    options: CompletePrismMeasureOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CompletePrismMeasureInspection, CompletePrismMeasureFailure> {
    let binding = capture_current_prism_binding(bridge, sources, receipt, &mut cancelled)?;
    let measures = measure_bound_prism(
        &binding,
        requested_extrusion(receipt),
        options,
        &mut cancelled,
    )?;
    poll_prism_binding(&mut cancelled)?;
    rebind_current_prism_binding(bridge, sources, receipt, &binding)?;
    Ok(CompletePrismMeasureInspection { binding, measures })
}
/// Integrate captured operands without entering mutable geometry; `measure_bound_prism(binding, extrusion, options, cancel)`.
/// The caller binds the supplied extrusion to the captured charts; this supplies scalar evidence only.
pub(crate) fn measure_bound_prism(
    binding: &CurrentPrismBinding,
    extrusion: [f64; 3],
    requested: CompletePrismMeasureOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<CompletePrismMeasures, BoundPrismMeasureFailure> {
    let subset = bound_subset(binding, extrusion, requested.subset, cancelled)?;
    let side = side::measure_side(binding, requested.side_area, cancelled)?;
    let total = total_measure(&subset, &side, requested.total_area, cancelled)?;
    Ok(CompletePrismMeasures {
        subset,
        side,
        total,
        requested,
    })
}

fn bound_subset(
    binding: &CurrentPrismBinding,
    extrusion: [f64; 3],
    options: PrismMeasureOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PrismMeasureSubset, BoundPrismMeasureFailure> {
    scalars::measure_subset(
        &binding.region,
        &binding.charts,
        extrusion,
        options,
        cancelled,
    )
    .map_err(BoundPrismMeasureFailure::Subset)
}
fn total_measure(
    subset: &PrismMeasureSubset,
    side: &CertifiedPolynomialSideIntegral,
    options: EnclosureSumOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<CertifiedEnclosureSum, BoundPrismMeasureFailure> {
    let components = [
        subset.first_cap.enclosure(),
        subset.last_cap.enclosure(),
        side.enclosure(),
    ];
    sum_nonnegative_enclosures(&components, options, cancelled)
        .map_err(BoundPrismMeasureFailure::Total)
}
