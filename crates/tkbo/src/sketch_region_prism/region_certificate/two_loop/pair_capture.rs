// SPDX-License-Identifier: GPL-2.0-only
//! Consume the existing complete source-bound matrix; never solve another pair query.
use super::{failure::*, pair_types::*, work::*};
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_certification::{
    BoundaryCensus, BoundaryDomain, CertificationObligation, DomainEvidence, domain_certificate,
};
use rustycad_tkmath::predicates::roots::{
    RootIsolation, bezier_pair::*, signs::RootSignEvaluation,
};

pub(super) fn domains<'a>(
    census: &'a BoundaryCensus,
    names: [RegionSourceId; 2],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopPairProof>> {
    work.count(
        "complete two-loop pair/self matrix",
        census.domains.len(),
        3,
    )?;
    super::census_budget::admit(census, work)?;
    let mut rows = Vec::new();
    for domain in &census.domains {
        rows.push(domain_row(census, domain, names, work)?);
    }
    let gaps = census.current_obligations();
    work.holds(
        "current exhaustive source-bound pair matrix",
        !gaps.contains(&CertificationObligation::ExhaustivePairAndSelfDomains)
            && !gaps.contains(&CertificationObligation::FailedWorkAccounting),
    )?;
    rows.sort_by_key(|row| row.sources);
    Ok(rows)
}
fn domain_row<'a>(
    census: &'a BoundaryCensus,
    domain: &BoundaryDomain,
    names: [RegionSourceId; 2],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopPairProof> {
    let proof = required_domain(census, domain, work)?;
    let [first, second] = domain.occurrences;
    work.units("pair proof", 1)?;
    let denominators = proof.denominators();
    Ok(TwoLoopPairProof {
        sources: [names[first], names[second]],
        policy: policy(census, domain),
        work: proof.work(),
        denominators: [
            isolation(denominators[0], work)?,
            isolation(denominators[1], work)?,
        ],
        homogeneous: homogeneous(proof.homogeneous_sources(), work)?,
        evidence: super::pair_graph::evidence(&proof, work)?,
        closure: closure(&proof, work)?,
    })
}
fn policy(census: &BoundaryCensus, domain: &BoundaryDomain) -> BezierPairPolicy {
    let DomainEvidence::PairedReport { report, .. } = domain.evidence else {
        unreachable!("genuine complete pair proof")
    };
    census
        .paired
        .as_ref()
        .expect("complete pair evidence")
        .reports[report]
        .effective
}
fn refusal<'a>(
    census: &'a BoundaryCensus,
    domain: &BoundaryDomain,
    work: &TwoLoopWork<'_>,
) -> Box<TwoLoopRegionFailure<'a>> {
    let mut failure = work.failure(TwoLoopRegionCause::Missing {
        hypothesis: "complete source-bound pair certificate",
    });
    if let DomainEvidence::PairedReport { report, .. } = domain.evidence
        && let Some(retained) = census.paired.as_ref().and_then(|p| p.reports.get(report))
    {
        match &retained.outcome {
            Ok(report) => {
                failure.cause = report_cause(report);
                failure.local = Some(TwoLoopLocalDiagnostic::Pair(report));
            }
            Err(cause) => failure.cause = TwoLoopRegionCause::PairCapture(cause.clone()),
        }
    }
    failure
}
fn homogeneous<'a>(
    source: [[ExactPairPolynomial<'_>; 3]; 2],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, [[TwoLoopExactPolynomial; 3]; 2]> {
    Ok([
        homogeneous_row(source[0], work)?,
        homogeneous_row(source[1], work)?,
    ])
}
fn homogeneous_row<'a>(
    source: [ExactPairPolynomial<'_>; 3],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, [TwoLoopExactPolynomial; 3]> {
    Ok([
        polynomial(source[0], work)?,
        polynomial(source[1], work)?,
        polynomial(source[2], work)?,
    ])
}
pub(super) fn polynomial<'a>(
    source: ExactPairPolynomial<'_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopExactPolynomial> {
    work.units("exact polynomial coefficient", source.coefficient_count())?;
    let coefficients = (0..source.coefficient_count())
        .map(|i| {
            source
                .signed_coefficient(i)
                .expect("retained coefficient")
                .to_string()
        })
        .collect();
    Ok(TwoLoopExactPolynomial(coefficients))
}
pub(super) fn isolation<'a>(
    source: &RootIsolation,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, RootIsolation> {
    work.units("root isolation", 1)?;
    work.units("root bracket", source.roots().len())?;
    Ok(source.clone())
}
pub(super) fn signs<'a>(
    source: &RootSignEvaluation,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, RootSignEvaluation> {
    work.units("root sign evaluation", 1)?;
    work.units("root sign bracket", source.isolation().roots().len())?;
    work.units("root sign", source.signs().len())?;
    Ok(source.clone())
}
fn closure<'a>(
    proof: &BezierPairCertificate<'_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Option<TwoLoopClosureProof>> {
    let Some(contact) = proof.contact(0) else {
        return Ok(None);
    };
    work.holds(
        "actual exact native closure at t=0,u=1",
        super::super::source::exact_closure(contact),
    )?;
    let contact = proof.contact(0).expect("retained closure");
    work.units("closure contact", 1)?;
    Ok(Some(TwoLoopClosureProof {
        parameter: contact.parameter.clone(),
        exact_paired_parameter: [1, 1],
        paired_parameter_bounds: contact.paired_parameter_bounds,
        world_bounds: contact.world_bounds,
        point: contact.point,
        achieved_world_error: contact.achieved_world_error,
    }))
}

fn required_domain<'a>(
    census: &'a BoundaryCensus,
    domain: &BoundaryDomain,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, BezierPairCertificate<'a>> {
    let proof = domain_certificate(census, domain).ok_or_else(|| refusal(census, domain, work))?;
    let [first, second] = domain.occurrences;
    work.holds(
        "pair source indices in complete two-source roster",
        first < 2 && second < 2,
    )?;
    work.count(
        "closure-only self / empty cross domain",
        proof.contact_count(),
        usize::from(first == second),
    )?;
    Ok(proof)
}

fn report_cause(report: &BezierPairReport) -> TwoLoopRegionCause {
    if report.completion() == BezierPairCompletion::Complete {
        TwoLoopRegionCause::Stale {
            operand: "retained complete pair source/policy binding",
        }
    } else {
        TwoLoopRegionCause::PairRefusal {
            completion: report.completion(),
            policy: report.policy(),
            work: report.work(),
        }
    }
}
