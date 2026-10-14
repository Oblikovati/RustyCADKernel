// SPDX-License-Identifier: GPL-2.0-only
//! ADR0051: apply the same injective affine product to all four bound annular face domains.
use super::super::{
    annular_binding::AnnularChartIdentity,
    hypothesis::{HypothesisCheck, require},
};
use super::{PrismIncidenceConclusion, PrismTrimRole, annular_domains::*, trim_incidence};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum AnnularIncidenceConclusion {
    InjectiveAnnularDomain,
    DisjointCaps,
    DisjointCertifiedBoundaries,
    TrimIncidence {
        first: Box<AnnularAuthoredTrim>,
        second: Box<AnnularAuthoredTrim>,
        conclusion: Box<PrismIncidenceConclusion>,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct AnnularFaceIncidence {
    pub first: AnnularChartIdentity,
    pub second: AnnularChartIdentity,
    pub conclusion: AnnularIncidenceConclusion,
}

/// Construct complete pair claims from bound domains; `ten_rows(domains)` alone grants no embedding authority.
/// The issuer also requires both regularity proofs, exact nonzero Jacobian and callback-free final rebind.
pub(super) fn ten_rows(domains: &AnnularDomains) -> HypothesisCheck<[AnnularFaceIncidence; 10]> {
    let AnnularDomains {
        first,
        last,
        outer,
        inner,
    } = domains;
    use AnnularIncidenceConclusion::*;
    Ok([
        row(first, first, InjectiveAnnularDomain),
        row(last, last, InjectiveAnnularDomain),
        row(outer, outer, seam(outer)?),
        row(inner, inner, seam(inner)?),
        row(first, last, DisjointCaps),
        row(first, outer, rim(first, outer, PrismTrimRole::First, 0.)?),
        row(first, inner, rim(first, inner, PrismTrimRole::First, 0.)?),
        row(last, outer, rim(last, outer, PrismTrimRole::Last, 1.)?),
        row(last, inner, rim(last, inner, PrismTrimRole::Last, 1.)?),
        row(outer, inner, DisjointCertifiedBoundaries),
    ])
}
fn seam(side: &AnnularFaceDomain) -> HypothesisCheck<AnnularIncidenceConclusion> {
    let start = trim(side, PrismTrimRole::Start, None)?;
    let end = trim(side, PrismTrimRole::End, None)?;
    let conclusion = trim_incidence::seam_conclusion(&start.operand, &end.operand)?;
    Ok(retain_incidence(start, end, conclusion))
}
fn rim(
    cap: &AnnularFaceDomain,
    side: &AnnularFaceDomain,
    role: PrismTrimRole,
    sweep: f64,
) -> HypothesisCheck<AnnularIncidenceConclusion> {
    let source = &side.identity.source;
    let cap_trim = trim(cap, role, Some(source))?;
    let side_trim = trim(side, role, Some(source))?;
    let conclusion = trim_incidence::rim_conclusion(&cap_trim.operand, &side_trim.operand, sweep)?;
    Ok(retain_incidence(cap_trim, side_trim, conclusion))
}
fn trim<'a>(
    domain: &'a AnnularFaceDomain,
    role: PrismTrimRole,
    source: Option<&crate::sketch_region::RegionObservedContext>,
) -> HypothesisCheck<&'a AnnularAuthoredTrim> {
    let matches: Vec<_> = domain
        .trims
        .iter()
        .filter(|trim| {
            trim.identity.role == role && source.is_none_or(|key| key == &trim.identity.source)
        })
        .collect();
    require(
        "annular incidence unique full source and directed role",
        matches.len(),
        1,
    )?;
    Ok(matches[0])
}
fn retain_incidence(
    first: &AnnularAuthoredTrim,
    second: &AnnularAuthoredTrim,
    conclusion: PrismIncidenceConclusion,
) -> AnnularIncidenceConclusion {
    AnnularIncidenceConclusion::TrimIncidence {
        first: Box::new(first.clone()),
        second: Box::new(second.clone()),
        conclusion: Box::new(conclusion),
    }
}
fn row(
    first: &AnnularFaceDomain,
    second: &AnnularFaceDomain,
    conclusion: AnnularIncidenceConclusion,
) -> AnnularFaceIncidence {
    AnnularFaceIncidence {
        first: first.identity.clone(),
        second: second.identity.clone(),
        conclusion,
    }
}
