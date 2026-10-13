// SPDX-License-Identifier: GPL-2.0-only
//! Enumerate the complete supplied occurrence universe before admitting any numerical pair.
use super::outcome::*;
use crate::sketch_region_candidates::CandidateSegment;
use rustycad_tkmath::predicates::roots::bezier_line::*;

/// Retain all cross/self domains; `enumerate_boundary_census(inputs, policy, || false)`.
/// Reports certify stored coefficients only, not projection, search or mapped split correctness.
pub(crate) fn enumerate_boundary_census(
    inputs: BoundaryInputs,
    policy: CensusPolicy,
    mut cancelled: impl FnMut() -> bool,
) -> Result<BoundaryCensus, CensusCause> {
    validate_policy(policy).map_err(CensusCause::Policy)?;
    let mut census = empty_census(inputs, policy);
    for pair in pair_domains(census.inputs.segments.len()) {
        if cancelled() {
            return Err(cancelled_census(census, pair, false));
        }
        let evidence = append_domain(&mut census, pair, &mut cancelled);
        if numerically_cancelled(&census, evidence) {
            return Err(cancelled_census(census, pair, true));
        }
    }
    Ok(census)
}
fn empty_census(inputs: BoundaryInputs, requested: CensusPolicy) -> BoundaryCensus {
    BoundaryCensus {
        inputs,
        domains: Vec::new(),
        reports: Vec::new(),
        failures: Vec::new(),
        requested,
        bisections: 0,
        missing: obligations(),
    }
}
fn append_domain(
    census: &mut BoundaryCensus,
    pair: [usize; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> DomainEvidence {
    let evidence = evaluate_domain(census, pair, cancelled);
    census.domains.push(BoundaryDomain {
        occurrences: pair,
        evidence,
    });
    evidence
}

fn evaluate_domain(
    census: &mut BoundaryCensus,
    pair: [usize; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> DomainEvidence {
    if let Some(gap) = unsupported_domain(census, pair) {
        return DomainEvidence::Unsupported(gap);
    }
    let Some((curve, line)) = line_pair(&census.inputs.segments, pair) else {
        return DomainEvidence::Unsupported(CensusGap::BezierPair);
    };
    evaluate_stored_pair(census, curve, line, cancelled)
}
fn unsupported_domain(census: &BoundaryCensus, pair: [usize; 2]) -> Option<CensusGap> {
    if pair[0] == pair[1] {
        return Some(CensusGap::SelfOffDiagonal);
    }
    let segments = &census.inputs.segments;
    segments[pair[0]]
        .edge
        .is_same(&segments[pair[1]].edge)
        .then_some(CensusGap::SamePhysicalCurve)
}
fn pair_domains(count: usize) -> impl Iterator<Item = [usize; 2]> {
    // Include the canonical self domain once; source directions remain separate occurrences.
    (0..count).flat_map(move |first| (first..count).map(move |second| [first, second]))
}

fn line_pair(segments: &[CandidateSegment], pair: [usize; 2]) -> Option<(usize, usize)> {
    if finite_line(&segments[pair[0]]) && finite_line(&segments[pair[1]]) {
        return Some(
            if physical_rank(segments, pair[0]) < physical_rank(segments, pair[1]) {
                (pair[0], pair[1])
            } else {
                (pair[1], pair[0])
            },
        );
    }
    if finite_line(&segments[pair[1]]) {
        return Some((pair[0], pair[1]));
    }
    if finite_line(&segments[pair[0]]) {
        return Some((pair[1], pair[0]));
    }
    None
}
fn finite_line(segment: &CandidateSegment) -> bool {
    segment.controls.len() == 2
        && segment.weights.is_none()
        && segment.interval == [0., 1.]
        && segment.controls[0] != segment.controls[1]
}
fn evaluate_stored_pair(
    census: &mut BoundaryCensus,
    curve: usize,
    line: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> DomainEvidence {
    if let Some(evidence) = reused_evidence(census, curve, line) {
        return evidence;
    }
    let result = certify_bezier_line_incidences(query(census, curve, line), cancelled);
    match result {
        Ok(report) => retain_report(census, report, curve, line),
        Err(failure) => retain_failure(census, failure),
    }
}
fn query(census: &BoundaryCensus, curve: usize, line: usize) -> BezierIncidenceQuery<'_> {
    BezierIncidenceQuery {
        curve_id: census.inputs.segments[curve].occurrence,
        line_id: census.inputs.segments[line].occurrence,
        curve: stored_query(census, curve, line),
        domain: BezierLineDomain::Segment,
        domain_sign: BezierDomainSignOptions::source_basis(),
        max_world_error: census.requested.max_world_error,
    }
}
fn stored_query(census: &BoundaryCensus, curve: usize, line: usize) -> BezierLineQuery<'_> {
    let source = &census.inputs.segments[curve];
    let line = &census.inputs.segments[line];
    BezierLineQuery {
        controls: &source.controls,
        weights: source.weights.as_deref(),
        line: [line.controls[0], line.controls[1]],
        interval: source.interval,
        options: census.requested.roots,
    }
}
fn reused_evidence(census: &BoundaryCensus, curve: usize, line: usize) -> Option<DomainEvidence> {
    census
        .domains
        .iter()
        .find_map(|domain| reused_domain(census, domain, (curve, line)))
}
fn reused_domain(
    census: &BoundaryCensus,
    domain: &BoundaryDomain,
    pair: (usize, usize),
) -> Option<DomainEvidence> {
    let old = line_pair(&census.inputs.segments, domain.occurrences)?;
    if !same_pair(&census.inputs.segments, pair, old) {
        return None;
    }
    match domain.evidence {
        DomainEvidence::StoredIncidence { report, .. } => Some(DomainEvidence::StoredIncidence {
            report,
            curve: pair.0,
            line: pair.1,
        }),
        DomainEvidence::NumericalFailure(index) => Some(DomainEvidence::NumericalFailure(index)),
        _ => None,
    }
}

fn same_pair(segments: &[CandidateSegment], first: (usize, usize), second: (usize, usize)) -> bool {
    same_capture(&segments[first.0], &segments[second.0])
        && same_capture(&segments[first.1], &segments[second.1])
}
fn same_capture(first: &CandidateSegment, second: &CandidateSegment) -> bool {
    first.edge.is_same(&second.edge)
        && equal_bits(
            first.controls.iter().flatten(),
            second.controls.iter().flatten(),
        )
        && first.weights.is_some() == second.weights.is_some()
        && equal_bits(
            first.weights.iter().flatten(),
            second.weights.iter().flatten(),
        )
        && equal_bits(first.interval.iter(), second.interval.iter())
        && equal_bits(
            first.original_interval.iter(),
            second.original_interval.iter(),
        )
}
fn equal_bits<'a>(
    first: impl Iterator<Item = &'a f64>,
    second: impl Iterator<Item = &'a f64>,
) -> bool {
    // Signed zero and NaN payloads remain bound to the exact captured descriptor.
    first
        .map(|value| value.to_bits())
        .eq(second.map(|value| value.to_bits()))
}

fn retain_report(
    census: &mut BoundaryCensus,
    report: BezierIncidenceReport,
    curve: usize,
    line: usize,
) -> DomainEvidence {
    let index = census.reports.len();
    census.bisections += u64::from(report.bisections());
    census.reports.push(report);
    DomainEvidence::StoredIncidence {
        report: index,
        curve,
        line,
    }
}
fn retain_failure(census: &mut BoundaryCensus, failure: BezierIncidenceFailure) -> DomainEvidence {
    // The provider retains typed failure but not total failed work: no aggregate budget claim.
    let index = census.failures.len();
    census.failures.push(failure);
    DomainEvidence::NumericalFailure(index)
}

fn obligations() -> Vec<CertificationObligation> {
    use CertificationObligation::*;
    vec![
        SourceConversionAndPlane,
        ExhaustivePairAndSelfDomains,
        FailedWorkAccounting,
        MappedSplitCorrespondence,
        CandidateSearchAndCoverage,
        SelfIntersection,
        Tolerance,
    ]
}

fn physical_rank(segments: &[CandidateSegment], index: usize) -> usize {
    // Operation-local first occurrence is deterministic; it is not a persistent source name.
    segments
        .iter()
        .position(|segment| same_capture(segment, &segments[index]))
        .unwrap_or(index)
}

fn numerically_cancelled(census: &BoundaryCensus, evidence: DomainEvidence) -> bool {
    match evidence {
        DomainEvidence::NumericalFailure(index) => {
            census.failures[index].completion() == BezierIncidenceCompletion::Cancelled
        }
        _ => false,
    }
}

fn cancelled_census(
    completed: BoundaryCensus,
    next_domain: [usize; 2],
    during_domain: bool,
) -> CensusCause {
    CensusCause::Cancelled {
        next_domain,
        during_domain,
        completed: Box::new(completed),
    }
}
fn validate_policy(policy: CensusPolicy) -> Result<(), CensusPolicyViolation> {
    if !policy.max_world_error.is_finite() || policy.max_world_error <= 0. {
        return Err(CensusPolicyViolation::WorldError {
            received: policy.max_world_error,
        });
    }
    if policy.roots.version != 1 {
        return Err(CensusPolicyViolation::RootVersion {
            received: policy.roots.version,
        });
    }
    if !policy.roots.max_width.is_finite() || policy.roots.max_width <= 0. {
        return Err(CensusPolicyViolation::ParameterWidth {
            received: policy.roots.max_width,
        });
    }
    Ok(())
}
