// SPDX-License-Identifier: GPL-2.0-only
//! Lookup uses authored roles plus source and complete directed trim identity.
use super::*;

pub(super) fn chart(
    binding: &AnnularPrismBinding,
    role: PrismImageRole,
    source: Option<RegionSourceId>,
) -> HypothesisCheck<&AnnularBoundChart> {
    let matches: Vec<_> = binding
        .charts()
        .iter()
        .filter(|c| c.identity.role == role && c.source_boundary == source)
        .collect();
    require(
        "annular embedding unique source-keyed chart",
        matches.len(),
        1,
    )?;
    Ok(matches[0])
}
pub(super) fn boundary(
    binding: &AnnularPrismBinding,
    source: RegionSourceId,
) -> HypothesisCheck<&AnnularBoundaryBinding> {
    let matches: Vec<_> = binding
        .boundaries()
        .iter()
        .filter(|b| b.source == source)
        .collect();
    require(
        "annular embedding unique authored boundary",
        matches.len(),
        1,
    )?;
    Ok(matches[0])
}
pub(super) fn retain_trims(chart: &AnnularBoundChart) -> HypothesisCheck<Vec<AnnularAuthoredTrim>> {
    chart
        .chart
        .captured
        .trims
        .iter()
        .map(|trim| {
            let matches: Vec<_> = chart
                .trims
                .iter()
                .filter(|identity| {
                    identity.role == trim.role
                        && identity.source.occurrence == trim.source
                        && identity.output.occurrence == trim.output
                })
                .collect();
            require(
                "annular embedding unique full directed trim",
                matches.len(),
                1,
            )?;
            Ok(AnnularAuthoredTrim {
                identity: matches[0].clone(),
                operand: trim_incidence::retain_trim(&chart.chart, trim),
            })
        })
        .collect()
}
pub(super) fn seam_endpoints(
    boundary: &AnnularBoundaryBinding,
    trims: &[AnnularAuthoredTrim],
) -> HypothesisCheck {
    for (role, parameter) in [(PrismTrimRole::Start, 0.), (PrismTrimRole::End, 1.)] {
        let matches: Vec<_> = trims.iter().filter(|t| t.identity.role == role).collect();
        require("annular embedding one directed seam use", matches.len(), 1)?;
        let count = boundary
            .endpoints
            .iter()
            .filter(|endpoint| {
                endpoint.context == matches[0].identity.source && endpoint.parameter == parameter
            })
            .count();
        require(
            "annular embedding full native endpoint correspondence",
            count,
            1,
        )?;
    }
    Ok(())
}
