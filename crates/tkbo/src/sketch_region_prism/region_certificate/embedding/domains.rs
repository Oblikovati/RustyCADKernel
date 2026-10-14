// SPDX-License-Identifier: GPL-2.0-only
//! Name actual full trims by authored roles; no geometric ranking supplies identity.
use super::super::hypothesis::{HypothesisCheck as RegionCheck, holds, need, require};
use super::*;

pub(super) fn capture_domains(binding: &CurrentPrismBinding) -> RegionCheck<[PrismFaceDomain; 3]> {
    Ok([
        cap_domain(binding, PrismImageRole::First, 0.)?,
        cap_domain(binding, PrismImageRole::Last, 1.)?,
        side_domain(binding)?,
    ])
}
pub(super) fn chart(
    binding: &CurrentPrismBinding,
    role: PrismImageRole,
) -> RegionCheck<&BoundPrismChart> {
    let matches: Vec<_> = binding
        .charts
        .iter()
        .filter(|chart| chart.role == role)
        .collect();
    require("embedding unique authored chart role", matches.len(), 1)?;
    Ok(matches[0])
}
fn cap_domain(
    binding: &CurrentPrismBinding,
    role: PrismImageRole,
    sweep: f64,
) -> RegionCheck<PrismFaceDomain> {
    let chart = chart(binding, role)?;
    holds(
        "embedding cap belongs to current Jordan component",
        chart.source == binding.region.copied.face,
    )?;
    let kind = PrismDomainKind::Cap {
        region: binding.region.copied.face.clone(),
        winding: binding.region.selected_winding,
        sweep,
    };
    Ok(retain_domain(chart, kind))
}
fn side_domain(binding: &CurrentPrismBinding) -> RegionCheck<PrismFaceDomain> {
    let chart = chart(binding, PrismImageRole::Generated)?;
    let height = need(
        chart.proof.side_height,
        "embedding exact stored side height",
    )?;
    bind_seam_endpoints(binding, chart)?;
    let kind = PrismDomainKind::Side {
        boundary: chart.source.clone(),
        native: [0., 1.],
        stored_sweep: [-height, 0.],
        normalized_sweep: [0., 1.],
        height,
    };
    Ok(retain_domain(chart, kind))
}
fn bind_seam_endpoints(binding: &CurrentPrismBinding, chart: &BoundPrismChart) -> RegionCheck {
    let endpoints = &binding.region.copied.boundaries[0].endpoints;
    for (role, parameter) in [(PrismTrimRole::Start, 0.), (PrismTrimRole::End, 1.)] {
        let trims: Vec<_> = chart
            .captured
            .trims
            .iter()
            .filter(|trim| trim.role == role)
            .collect();
        require("embedding unique authored seam use", trims.len(), 1)?;
        let matches = endpoints
            .iter()
            .filter(|(key, t)| *key == trims[0].source && *t == parameter)
            .count();
        require("embedding seam uses current closure endpoint", matches, 1)?;
    }
    Ok(())
}
fn retain_domain(chart: &BoundPrismChart, kind: PrismDomainKind) -> PrismFaceDomain {
    PrismFaceDomain {
        face: chart.output.clone(),
        source: chart.source.clone(),
        role: chart.role,
        parents: chart.parents.clone(),
        kind,
        trims: chart
            .captured
            .trims
            .iter()
            .map(|trim| retain_trim(chart, trim))
            .collect(),
    }
}
fn retain_trim(chart: &BoundPrismChart, trim: &CapturedPrismTrim) -> PrismAuthoredTrimUse {
    PrismAuthoredTrimUse {
        face: chart.output.clone(),
        output: trim.output.clone(),
        source: trim.source.clone(),
        parents: trim.parents.clone(),
        role: trim.role,
        interval: trim.interval,
    }
}
