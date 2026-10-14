// SPDX-License-Identifier: GPL-2.0-only
//! J != 0 makes F(u,v,s)=A(u,v)+sE injective; apply only to the captured Jordan prism.
use super::super::hypothesis::{HypothesisCheck as RegionCheck, require};
use super::*;

pub(super) fn six_rows(domains: &[PrismFaceDomain; 3]) -> RegionCheck<[PrismFaceIncidence; 6]> {
    let first = domain(domains, PrismImageRole::First)?;
    let last = domain(domains, PrismImageRole::Last)?;
    let side = domain(domains, PrismImageRole::Generated)?;
    Ok([
        row(
            first,
            first,
            PrismIncidenceConclusion::InjectivePlanarDomain,
        ),
        row(last, last, PrismIncidenceConclusion::InjectivePlanarDomain),
        row(side, side, seam(side)?),
        row(first, last, PrismIncidenceConclusion::Disjoint),
        row(first, side, rim(first, side, PrismTrimRole::First, 0.)?),
        row(last, side, rim(last, side, PrismTrimRole::Last, 1.)?),
    ])
}
fn domain(domains: &[PrismFaceDomain], role: PrismImageRole) -> RegionCheck<&PrismFaceDomain> {
    let matches: Vec<_> = domains
        .iter()
        .filter(|domain| domain.role == role)
        .collect();
    require("embedding unique trimmed face domain", matches.len(), 1)?;
    Ok(matches[0])
}
fn authored_trim(
    domain: &PrismFaceDomain,
    role: PrismTrimRole,
) -> RegionCheck<&PrismAuthoredTrimUse> {
    let matches: Vec<_> = domain
        .trims
        .iter()
        .filter(|trim| trim.role == role)
        .collect();
    require("embedding unique authored trim role", matches.len(), 1)?;
    Ok(matches[0])
}
fn seam(side: &PrismFaceDomain) -> RegionCheck<PrismIncidenceConclusion> {
    let start = authored_trim(side, PrismTrimRole::Start)?;
    let end = authored_trim(side, PrismTrimRole::End)?;
    trim_incidence::seam_conclusion(start, end)
}

fn rim(
    cap: &PrismFaceDomain,
    side: &PrismFaceDomain,
    role: PrismTrimRole,
    sweep: f64,
) -> RegionCheck<PrismIncidenceConclusion> {
    let cap_trim = authored_trim(cap, role)?;
    let side_trim = authored_trim(side, role)?;
    trim_incidence::rim_conclusion(cap_trim, side_trim, sweep)
}

fn row(
    first: &PrismFaceDomain,
    second: &PrismFaceDomain,
    conclusion: PrismIncidenceConclusion,
) -> PrismFaceIncidence {
    PrismFaceIncidence {
        first: first.clone(),
        second: second.clone(),
        conclusion,
    }
}
