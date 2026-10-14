// SPDX-License-Identifier: GPL-2.0-only
use super::*;

/// Capture four actual domains; `capture_annular_domains(binding)` still requires regularity and incidence gates.
pub(in super::super) fn capture_annular_domains(
    binding: &AnnularPrismBinding,
) -> HypothesisCheck<AnnularDomains> {
    require(
        "annular embedding complete chart census",
        binding.charts().len(),
        4,
    )?;
    let nesting = binding.region().nesting();
    Ok(AnnularDomains {
        first: cap(binding, PrismImageRole::First, 0.)?,
        last: cap(binding, PrismImageRole::Last, 1.)?,
        outer: side(binding, nesting.outer)?,
        inner: side(binding, nesting.inner)?,
    })
}
fn cap(
    binding: &AnnularPrismBinding,
    role: PrismImageRole,
    sweep: f64,
) -> HypothesisCheck<AnnularFaceDomain> {
    let chart = selection::chart(binding, role, None)?;
    let nesting = binding.region().nesting();
    let trims = selection::retain_trims(chart)?;
    require("annular embedding both directed cap rims", trims.len(), 2)?;
    Ok(AnnularFaceDomain {
        identity: chart.identity.clone(),
        kind: AnnularDomainKind::Cap {
            region: binding.region().selected_candidate_id(),
            outer: nesting.outer,
            inner: nesting.inner,
            normalized_sweep: sweep,
        },
        trims,
    })
}
fn side(
    binding: &AnnularPrismBinding,
    source: RegionSourceId,
) -> HypothesisCheck<AnnularFaceDomain> {
    let chart = selection::chart(binding, PrismImageRole::Generated, Some(source))?;
    let boundary = selection::boundary(binding, source)?;
    holds(
        "annular embedding full copied source context",
        chart.identity.source == boundary.copied,
    )?;
    let height = need(
        chart.chart.proof.side_height,
        "annular embedding exact side height",
    )?;
    Ok(AnnularFaceDomain {
        identity: chart.identity.clone(),
        kind: side_kind(boundary, height),
        trims: side_trims(chart, boundary)?,
    })
}
fn side_trims(
    chart: &AnnularBoundChart,
    boundary: &AnnularBoundaryBinding,
) -> HypothesisCheck<Vec<AnnularAuthoredTrim>> {
    let trims = selection::retain_trims(chart)?;
    require(
        "annular embedding complete side trim census",
        trims.len(),
        4,
    )?;
    selection::seam_endpoints(boundary, &trims)?;
    Ok(trims)
}
fn side_kind(boundary: &AnnularBoundaryBinding, height: f64) -> AnnularDomainKind {
    AnnularDomainKind::Side {
        source: boundary.source,
        boundary: boundary.copied.clone(),
        native: [0., 1.],
        stored_sweep: [-height, 0.],
        normalized_sweep: [0., 1.],
        height,
    }
}
