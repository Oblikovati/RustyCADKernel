// SPDX-License-Identifier: GPL-2.0-only
//! Jordan evidence from the complete census pair matrix: every certified contact is an
//! authored join at exact native endpoints, and every authored join is certified.
//! With continuous injective edges this makes each loop a simple closed curve and the
//! loops pairwise disjoint; no new intersection query is solved here.
use super::binding::SegmentBinding;
use super::types::*;
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_certification::{
    BoundaryCensus, CertificationObligation, domain_certificate,
};
use rustycad_tkmath::predicates::roots::bezier_pair::{
    BezierPairSignStage, CertifiedBezierPairContact,
};
use std::collections::BTreeMap;

const STAGE: LoopRegionStage = LoopRegionStage::Joins;
/// Expected endpoint parameters per census domain `(low, high)` position pair.
pub(super) type Expected = BTreeMap<(usize, usize), Vec<[u8; 2]>>;

/// Certified joins and pair-domain counts `(joins, carrier pairs, domains)`.
pub(super) fn certify(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
) -> LoopRegionCheck<(Vec<JoinProof>, usize, usize)> {
    let expected = expected_joins(request, binding);
    certify_contacts(&request.bridge.census, binding, expected)
}
/// Every certified contact of the complete pair matrix must be one of `expected` (exact
/// native endpoints), and every expected contact must be certified; shared by the loop
/// proof (authored joins) and the plane graph proof (vertex incidences, ADR 0084).
pub(super) fn certify_contacts(
    census: &BoundaryCensus,
    binding: &SegmentBinding<'_>,
    mut expected: Expected,
) -> LoopRegionCheck<(Vec<JoinProof>, usize, usize)> {
    domain_count(census, binding.sources.len())?;
    first_unavailable(census, binding)?;
    current_matrix(census)?;
    let mut joins = Vec::new();
    let mut carriers = 0;
    for domain in &census.domains {
        let [first, second] = domain.occurrences;
        let sources = [source(binding, first)?, source(binding, second)?];
        let unavailable = LoopRegionFailure {
            stage: STAGE,
            cause: LoopRegionCause::PairUnavailable { sources },
        };
        let proof = domain_certificate(census, domain).ok_or(unavailable)?;
        carriers += usize::from(proof.sign_stage() == BezierPairSignStage::SkippedCommonCarrier);
        let wanted = expected.remove(&key(first, second)).unwrap_or_default();
        let wanted = oriented(wanted, first > second);
        let found = (0..proof.contact_count()).map(|i| proof.contact(i).expect("indexed contact"));
        joins.extend(match_contacts(sources, found, wanted)?);
    }
    if let Some(((low, high), parameters)) = expected.into_iter().next() {
        let sources = [source(binding, low)?, source(binding, high)?];
        let missing = LoopRegionCause::MissingJoin {
            sources,
            parameter: parameters[0],
        };
        return refuse(STAGE, missing);
    }
    Ok((joins, carriers, census.domains.len()))
}
fn domain_count(census: &BoundaryCensus, sources: usize) -> LoopRegionCheck<()> {
    let expected = sources * (sources + 1) / 2;
    if census.domains.len() != expected {
        let actual = census.domains.len();
        let quantity = "complete pair and self domains";
        return refuse(
            STAGE,
            LoopRegionCause::Count {
                quantity,
                actual,
                expected,
            },
        );
    }
    Ok(())
}
/// Name the first pair domain without a complete certificate, before matrix obligations.
fn first_unavailable(census: &BoundaryCensus, binding: &SegmentBinding<'_>) -> LoopRegionCheck<()> {
    for domain in &census.domains {
        if domain_certificate(census, domain).is_none() {
            let [first, second] = domain.occurrences;
            let sources = [source(binding, first)?, source(binding, second)?];
            return refuse(STAGE, LoopRegionCause::PairUnavailable { sources });
        }
    }
    Ok(())
}
fn current_matrix(census: &BoundaryCensus) -> LoopRegionCheck<()> {
    let gaps = census.current_obligations();
    let open = gaps.contains(&CertificationObligation::ExhaustivePairAndSelfDomains)
        || gaps.contains(&CertificationObligation::FailedWorkAccounting);
    if open {
        let quantity = "current exhaustive source-bound pair matrix";
        return refuse(STAGE, LoopRegionCause::Obligation { quantity });
    }
    Ok(())
}
fn source(binding: &SegmentBinding<'_>, position: usize) -> LoopRegionCheck<RegionSourceId> {
    binding.source_at(position).ok_or(LoopRegionFailure {
        stage: STAGE,
        cause: LoopRegionCause::Count {
            quantity: "bound census position",
            actual: position,
            expected: 0,
        },
    })
}
pub(super) fn key(first: usize, second: usize) -> (usize, usize) {
    (first.min(second), first.max(second))
}
/// Swap expected parameter pairs when the domain lists the higher position first.
fn oriented(parameters: Vec<[u8; 2]>, swapped: bool) -> Vec<[u8; 2]> {
    if !swapped {
        return parameters;
    }
    parameters.into_iter().map(|[a, b]| [b, a]).collect()
}
/// Each consecutive pair `A -> B` joins `A`'s directed end to `B`'s directed start.
fn expected_joins(request: &LoopRegionRequest<'_, '_>, binding: &SegmentBinding<'_>) -> Expected {
    let mut expected = Expected::new();
    for authored in request.loops {
        let count = authored.uses.len();
        for position in 0..count {
            let (a, b) = (
                authored.uses[position],
                authored.uses[(position + 1) % count],
            );
            let (end, start) = (u8::from(!a.reversed), u8::from(b.reversed));
            let (pa, pb) = (binding.position(a.source), binding.position(b.source));
            // A closed single-edge loop is its canonical self contact t = 0 < u = 1.
            let entry = if pa == pb {
                [0, 1]
            } else if pa < pb {
                [end, start]
            } else {
                [start, end]
            };
            expected.entry(key(pa, pb)).or_default().push(entry);
        }
    }
    expected
}
fn match_contacts<'p>(
    sources: [RegionSourceId; 2],
    found: impl Iterator<Item = CertifiedBezierPairContact<'p>>,
    mut wanted: Vec<[u8; 2]>,
) -> LoopRegionCheck<Vec<JoinProof>> {
    let mut joins = Vec::new();
    for contact in found {
        let parameter = exact_endpoints(&contact);
        let index = parameter.and_then(|p| wanted.iter().position(|w| *w == p));
        let Some(index) = index else {
            let point = contact.point;
            return refuse(STAGE, LoopRegionCause::UnexpectedContact { sources, point });
        };
        joins.push(JoinProof {
            sources,
            parameter: wanted.swap_remove(index),
        });
    }
    match wanted.first() {
        Some(parameter) => refuse(
            STAGE,
            LoopRegionCause::MissingJoin {
                sources,
                parameter: *parameter,
            },
        ),
        None => Ok(joins),
    }
}
/// Both contact parameters as exact native endpoints, or `None`.
fn exact_endpoints(contact: &CertifiedBezierPairContact<'_>) -> Option<[u8; 2]> {
    let (lower, upper) = contact.parameter.bounds();
    let first = [0u8, 1].into_iter().find(|v| {
        let value = f64::from(*v);
        lower.compare_f64(value).is_ok_and(|o| o.is_eq())
            && upper.compare_f64(value).is_ok_and(|o| o.is_eq())
    })?;
    let paired = &contact.exact_paired_parameter;
    let second = [0u8, 1].into_iter().find(|v| {
        let value = f64::from(*v);
        paired.compare_lower(value).is_ok_and(|o| o.is_eq())
            && paired.compare_upper(value).is_ok_and(|o| o.is_eq())
    })?;
    Some([first, second])
}
