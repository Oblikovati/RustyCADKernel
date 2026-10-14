// SPDX-License-Identifier: GPL-2.0-only
//! Bind exact provider proofs to current standard evaluators and captured original revisions.
use super::{PairedContactTransaction, types::*};
use crate::sketch_region::{ArrangementSource, OriginalMemberIdentity};
use crate::sketch_region_certification::{
    BoundaryCensus, CertificationObligation, domain_certificate,
};
use crate::sketch_region_source_conformance::*;
use rustycad_tkbrep::topods::edge;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

pub(super) fn source_hypotheses(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
) -> RegionCheck<OriginalMemberIdentity> {
    let census = &bridge.census;
    require("one source occurrence", census.inputs.segments.len(), 1)?;
    require(
        "one census construction",
        census.inputs.constructions.len(),
        1,
    )?;
    require("one original boundary", census.inputs.originals.len(), 1)?;
    require("one original request", sources.len(), 1)?;
    current_standard_original(census, sources)?;
    current_standard_construction(census)?;
    exact_source_stages(census)?;
    closure_only(census)?;
    Ok(census.inputs.originals[0].original.clone())
}
fn current_standard_original(
    census: &BoundaryCensus,
    sources: &[ArrangementSource<'_>],
) -> RegionCheck {
    let actual = edge(sources[0].shape).map_err(RegionCertificateCause::Mapped)?;
    let captured = super::super::surface_charts::capture_bezier_edge(&actual)
        .map_err(|cause| RegionCertificateCause::Capture(std::sync::Arc::new(cause)))?;
    let original = &census.inputs.originals[0];
    holds(
        "original source identity",
        crate::sketch_region::original_identity(sources[0].identity, &actual) == original.original,
    )?;
    holds(
        "current original coefficients/range/placement",
        original
            .representation
            .as_ref()
            .is_ok_and(|old| same_space_representation(old, &captured)),
    )
}
fn current_standard_construction(census: &BoundaryCensus) -> RegionCheck {
    let segment = &census.inputs.segments[0];
    let actual = edge(&segment.edge).map_err(RegionCertificateCause::Mapped)?;
    super::super::surface_charts::capture_bezier_edge(&actual)
        .map_err(|cause| RegionCertificateCause::Capture(std::sync::Arc::new(cause)))?;
    super::super::surface_charts::pcurve_on(&segment.edge, &census.inputs.construction_support)
        .map_err(|cause| RegionCertificateCause::Capture(std::sync::Arc::new(cause)))?;
    holds(
        "full native census domain",
        segment.interval == [0., 1.] && segment.original_interval == [0., 1.],
    )?;
    holds(
        "source representation and complete current capture",
        census.source_representation_certified(),
    )
}
fn exact_source_stages(census: &BoundaryCensus) -> RegionCheck {
    let binding = need(census.source_binding.as_ref(), "source conformance binding")?;
    let entries = &binding.inspection().entries;
    require("one exact source construction", entries.len(), 1)?;
    require("all source construction stages", entries[0].stages.len(), 3)?;
    for stage in &entries[0].stages {
        holds(
            "zero exact source residual",
            matches!(&stage.decision,
            Ok(ResidualDecision::Within { proven_squared_upper })
            if proven_squared_upper.compare_f64(0.).is_ok_and(|order|order.is_eq())),
        )?;
    }
    Ok(())
}
fn closure_only(census: &BoundaryCensus) -> RegionCheck {
    require("one exhaustive self domain", census.domains.len(), 1)?;
    let obligations = census.current_obligations();
    holds(
        "complete source-bound self census",
        !obligations.contains(&CertificationObligation::ExhaustivePairAndSelfDomains)
            && !obligations.contains(&CertificationObligation::FailedWorkAccounting),
    )?;
    let proof = domain_certificate(census, &census.domains[0])
        .ok_or(RegionCertificateCause::UnavailableCandidates)?;
    require("only one off-diagonal contact", proof.contact_count(), 1)?;
    holds(
        "exact closure t=0,u=1",
        exact_closure(proof.contact(0).expect("one complete contact")),
    )
}
fn exact_closure(
    contact: rustycad_tkmath::predicates::roots::bezier_pair::CertifiedBezierPairContact<'_>,
) -> bool {
    let (lower, upper) = contact.parameter.bounds();
    let second = contact.exact_paired_parameter;
    lower.compare_f64(0.).is_ok_and(|order| order.is_eq())
        && upper.compare_f64(0.).is_ok_and(|order| order.is_eq())
        && second.compare_lower(1.).is_ok_and(|order| order.is_eq())
        && second.compare_upper(1.).is_ok_and(|order| order.is_eq())
}
