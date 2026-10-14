// SPDX-License-Identifier: GPL-2.0-only
use super::super::annular_regularity::AnnularRegularityOperand;
use super::*;
use crate::sketch_region_prism::surface_charts::CapturedChartSource;
use crate::sketch_region_source_conformance::{
    CapturedSpaceRepresentation, same_space_representation,
};

/// Supply certified outer/inner roles with actual copied chart coefficients; `bound_regularity_operands(binding)`.
pub(in super::super) fn bound_regularity_operands(
    binding: &AnnularPrismBinding,
) -> HypothesisCheck<[AnnularRegularityOperand<'_>; 2]> {
    let nesting = binding.region().nesting();
    Ok([
        operand(binding, nesting.outer)?,
        operand(binding, nesting.inner)?,
    ])
}
fn operand(
    binding: &AnnularPrismBinding,
    source: RegionSourceId,
) -> HypothesisCheck<AnnularRegularityOperand<'_>> {
    let chart = selection::chart(binding, PrismImageRole::Generated, Some(source))?;
    let boundary = selection::boundary(binding, source)?;
    holds(
        "annular regularity full copied source context",
        chart.identity.source == boundary.copied,
    )?;
    let space = actual_source(chart)?;
    check_copied_space(binding, boundary, space)?;
    Ok(AnnularRegularityOperand {
        source,
        curve_id: source.0,
        space,
    })
}
fn check_copied_space(
    binding: &AnnularPrismBinding,
    boundary: &AnnularBoundaryBinding,
    space: &CapturedSpaceRepresentation,
) -> HypothesisCheck {
    let copied: Vec<_> = binding
        .copied_sources()
        .boundaries
        .iter()
        .filter(|b| b.occurrence == boundary.copied.occurrence)
        .collect();
    require(
        "annular regularity unique actual copied curve",
        copied.len(),
        1,
    )?;
    holds(
        "annular regularity actual chart equals copied curve",
        same_space_representation(space, &copied[0].space),
    )
}
fn actual_source(chart: &AnnularBoundChart) -> HypothesisCheck<&CapturedSpaceRepresentation> {
    if let CapturedChartSource::Side(space) = &chart.chart.captured.source {
        return Ok(space);
    }
    Err(
        super::super::super::hypothesis::RegionHypothesisCause::Missing {
            hypothesis: "annular regularity actual generating side curve",
        },
    )
}
