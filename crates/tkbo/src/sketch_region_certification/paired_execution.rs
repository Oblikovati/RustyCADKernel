// SPDX-License-Identifier: GPL-2.0-only
//! Every cold solver execution spends the shared remaining budget, including failed and cancelled work.
use super::{census, outcome::*, paired_types::*};
use rustycad_tkmath::predicates::roots::bezier_pair::*;

pub(super) fn enumerate_paired_domains(
    mut census: BoundaryCensus,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BoundaryCensus, CensusCause> {
    for pair in census::pair_domains(census.inputs.segments.len()) {
        if cancelled() {
            return Err(stopped(census, pair, false));
        }
        let evidence = append_pair(&mut census, pair, cancelled);
        if evidence_cancelled(&census, evidence) {
            return Err(stopped(census, pair, true));
        }
    }
    super::paired_binding::refresh_obligations(&mut census);
    Ok(census)
}
fn append_pair(
    census: &mut BoundaryCensus,
    pair: [usize; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> DomainEvidence {
    let evidence = evaluate_pair(census, pair, cancelled);
    census.domains.push(BoundaryDomain {
        occurrences: pair,
        evidence,
    });
    evidence
}
fn evaluate_pair(
    census: &mut BoundaryCensus,
    pair: [usize; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> DomainEvidence {
    let sources = canonical_sources(census, pair);
    if pair[0] != pair[1] && sources[0] == sources[1] {
        return DomainEvidence::Unsupported(CensusGap::SamePhysicalCurve);
    }
    let mode = query_mode(pair);
    if let Some(report) = reused_report(census, sources, mode) {
        census.paired.as_mut().unwrap().reused_reports += 1;
        return report_reference(census, pair, report, true);
    }
    let report = retain_execution(census, sources, mode, cancelled);
    report_reference(census, pair, report, false)
}
fn canonical_sources(census: &BoundaryCensus, pair: [usize; 2]) -> [usize; 2] {
    let records = &census.paired.as_ref().unwrap().occurrences;
    let mut sources = pair.map(|index| records[index].physical_source);
    // Operation-local traversal rank canonicalizes execution only, never persistent names or geometry order.
    sources.sort();
    sources
}
fn reused_report(
    census: &BoundaryCensus,
    sources: [usize; 2],
    mode: PairedQueryMode,
) -> Option<usize> {
    census
        .paired
        .as_ref()
        .unwrap()
        .reports
        .iter()
        .position(|record| record.sources == sources && record.mode == mode)
}
fn retain_execution(
    census: &mut BoundaryCensus,
    sources: [usize; 2],
    mode: PairedQueryMode,
    cancelled: &mut impl FnMut() -> bool,
) -> usize {
    let record = cold_execution(census, sources, mode, cancelled);
    census.bisections += record.completed_work();
    let paired = census.paired.as_mut().unwrap();
    let report = paired.reports.len();
    paired.reports.push(record);
    report
}
fn cold_execution(
    census: &BoundaryCensus,
    sources: [usize; 2],
    mode: PairedQueryMode,
    cancelled: &mut impl FnMut() -> bool,
) -> PairedStoredReport {
    let effective = remaining_policy(census);
    let outcome =
        captured_query(census, sources, mode).map(|query| query.certify(effective, cancelled));
    PairedStoredReport {
        sources,
        mode,
        effective,
        outcome,
    }
}
pub(super) fn query_mode(pair: [usize; 2]) -> PairedQueryMode {
    if pair[0] == pair[1] {
        return PairedQueryMode::SelfOffDiagonal;
    }
    PairedQueryMode::Distinct
}
fn remaining_policy(census: &BoundaryCensus) -> BezierPairPolicy {
    let mut effective = census.paired.as_ref().unwrap().requested;
    let used = u16::try_from(census.bisections).unwrap_or(u16::MAX);
    effective.roots.max_bisections = effective.roots.max_bisections.saturating_sub(used);
    effective
}
pub(super) fn captured_query(
    census: &BoundaryCensus,
    sources: [usize; 2],
    mode: PairedQueryMode,
) -> Result<CapturedBezierPair, Box<BezierPairCaptureFailure>> {
    let views = sources.map(|index| source_view(&census.inputs.segments[index]));
    let query = match mode {
        PairedQueryMode::Distinct => CapturedBezierPair::capture_pair(views[0], views[1]),
        PairedQueryMode::SelfOffDiagonal => CapturedBezierPair::self_off_diagonal(
            views[0].controls,
            views[0].weights,
            views[0].interval,
        ),
    };
    query.map_err(Box::new)
}
fn source_view(source: &crate::sketch_region_candidates::CandidateSegment) -> BezierPairCurve<'_> {
    BezierPairCurve {
        controls: &source.controls,
        weights: source.weights.as_deref(),
        interval: source.interval,
    }
}
fn evidence_cancelled(census: &BoundaryCensus, evidence: DomainEvidence) -> bool {
    let DomainEvidence::PairedReport { report, .. } = evidence else {
        return false;
    };
    census.paired.as_ref().unwrap().reports[report].completion() == BezierPairCompletion::Cancelled
}
fn stopped(completed: BoundaryCensus, next_domain: [usize; 2], during_domain: bool) -> CensusCause {
    CensusCause::Cancelled {
        next_domain,
        during_domain,
        completed: Box::new(completed),
    }
}

fn report_reference(
    census: &BoundaryCensus,
    pair: [usize; 2],
    report: usize,
    reused: bool,
) -> DomainEvidence {
    let Some(source_slots) = parameter_slots(census, pair) else {
        return DomainEvidence::Unsupported(CensusGap::MissingOccurrence {
            occurrences: pair,
            count: census.inputs.segments.len(),
        });
    };
    DomainEvidence::PairedReport {
        report,
        reused,
        source_slots,
    }
}
pub(super) fn parameter_slots(census: &BoundaryCensus, pair: [usize; 2]) -> Option<[usize; 2]> {
    let records = &census.paired.as_ref().unwrap().occurrences;
    let first = records.get(pair[0])?;
    let second = records.get(pair[1])?;
    if pair[0] != pair[1] && first.physical_source > second.physical_source {
        return Some([1, 0]);
    }
    Some([0, 1])
}
