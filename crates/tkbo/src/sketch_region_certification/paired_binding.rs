// SPDX-License-Identifier: GPL-2.0-only
//! Coverage is recomputed from the actual complete occurrence matrix, not a mutable gap vector.
use super::{census, outcome::*, paired_types::*};
use rustycad_tkmath::predicates::roots::bezier_pair::*;

pub(super) fn refresh_obligations(census: &mut BoundaryCensus) {
    let complete = paired_domains_certified(census);
    let work = paired_work_exact(census);
    census.missing.retain(|gap| {
        !(*gap == CertificationObligation::ExhaustivePairAndSelfDomains && complete
            || *gap == CertificationObligation::FailedWorkAccounting && work)
    });
}
pub(super) fn paired_domains_certified(census: &BoundaryCensus) -> bool {
    let Some(paired) = &census.paired else {
        return false;
    };
    let expected = census::pair_domains(census.inputs.segments.len()).collect::<Vec<_>>();
    !expected.is_empty()
        && expected.len() == census.domains.len()
        && paired_policy_current(census, paired)
        && census.domains.iter().zip(expected).all(|(domain, pair)| {
            domain.occurrences == pair && domain_certificate(census, domain).is_some()
        })
}
pub(crate) fn domain_certificate<'a>(
    census: &'a BoundaryCensus,
    domain: &BoundaryDomain,
) -> Option<BezierPairCertificate<'a>> {
    let DomainEvidence::PairedReport { report, .. } = domain.evidence else {
        return None;
    };
    if !census.source_representation_certified() || !paired_work_exact(census) {
        return None;
    }
    let paired = census.paired.as_ref()?;
    let retained = paired.reports.get(report)?;
    if !paired_policy_current(census, paired) || !domain_binding_current(census, domain, retained) {
        return None;
    }
    let report = retained.outcome.as_ref().ok()?;
    (report.policy() == retained.effective)
        .then(|| report.complete())
        .flatten()
}
fn paired_policy_current(census: &BoundaryCensus, paired: &PairedCensusEvidence) -> bool {
    // The retained request records its own version; version 2 adds ADR 0067 carriers.
    (1..=2).contains(&paired.requested.version)
        && paired.requested.signs == BezierPairSignOptions::derived()
        && paired.requested.roots == census.requested.roots
        && paired.requested.requested_world_error.to_bits()
            == census.requested.max_world_error.to_bits()
}
fn domain_binding_current(
    census: &BoundaryCensus,
    domain: &BoundaryDomain,
    record: &PairedStoredReport,
) -> bool {
    if !parameter_binding_current(census, domain) {
        return false;
    }
    let pair = domain.occurrences;
    let Some(sources) = current_sources(census, pair) else {
        return false;
    };
    let mode = super::paired_execution::query_mode(pair);
    sources == record.sources
        && mode == record.mode
        && pair
            .into_iter()
            .all(|index| occurrence_current(census, index))
        && report_source_current(census, record)
}
fn current_sources(census: &BoundaryCensus, pair: [usize; 2]) -> Option<[usize; 2]> {
    let records = &census.paired.as_ref()?.occurrences;
    let mut sources = [
        records.get(pair[0])?.physical_source,
        records.get(pair[1])?.physical_source,
    ];
    sources.sort();
    Some(sources)
}
fn occurrence_current(census: &BoundaryCensus, index: usize) -> bool {
    let Some(record) = census
        .paired
        .as_ref()
        .and_then(|paired| paired.occurrences.get(index))
    else {
        return false;
    };
    let Some(current) = census.inputs.segments.get(index) else {
        return false;
    };
    same_occurrence_metadata(&record.source, current)
        && kinded_parents_complete(record)
        && current_relations(census, current, &record.relations)
        && current_physical_group(census, index, current) == Some(record.physical_source)
}
fn same_occurrence_metadata(
    old: &crate::sketch_region_candidates::CandidateSegment,
    current: &crate::sketch_region_candidates::CandidateSegment,
) -> bool {
    census::same_capture(old, current)
        && old.edge.is_equal(&current.edge)
        && old.parents == current.parents
        && old.occurrence == current.occurrence
        && old.reversed == current.reversed
}
fn current_physical_group(
    census: &BoundaryCensus,
    index: usize,
    current: &crate::sketch_region_candidates::CandidateSegment,
) -> Option<usize> {
    census.inputs.segments[..=index]
        .iter()
        .position(|source| census::same_capture(source, current))
}

fn current_relations(
    census: &BoundaryCensus,
    source: &crate::sketch_region_candidates::CandidateSegment,
    recorded: &[crate::sketch_region::ArrangementRelation],
) -> bool {
    let actual = census
        .inputs
        .relations
        .iter()
        .filter(|row| row.output.is_same(&source.edge))
        .collect::<Vec<_>>();
    actual.len() == recorded.len()
        && actual.iter().zip(recorded).all(|(a, b)| {
            a.output.is_equal(&b.output)
                && a.preserved == b.preserved
                && a.modified == b.modified
                && a.generated == b.generated
        })
}
fn report_source_current(census: &BoundaryCensus, record: &PairedStoredReport) -> bool {
    let Ok(report) = &record.outcome else {
        return false;
    };
    let Ok(expected) = super::paired_execution::captured_query(census, record.sources, record.mode)
    else {
        return false;
    };
    report.source() == &expected
}
pub(super) fn paired_work_exact(census: &BoundaryCensus) -> bool {
    let Some(paired) = &census.paired else {
        return false;
    };
    let mut used = 0;
    for record in &paired.reports {
        if !effective_work_current(paired, record, used) {
            return false;
        }
        used += record.completed_work();
    }
    used == census.bisections
        && paired_policy_current(census, paired)
        && reuse_work_current(census, paired)
}
fn effective_work_current(
    paired: &PairedCensusEvidence,
    record: &PairedStoredReport,
    used: u64,
) -> bool {
    let Some(remaining) = u64::from(paired.requested.roots.max_bisections).checked_sub(used) else {
        return false;
    };
    let mut expected = paired.requested;
    expected.roots.max_bisections = remaining as u16;
    record.effective == expected
        && record.completed_work() <= remaining
        && record
            .outcome
            .as_ref()
            .map_or(true, |report| report.policy() == expected)
}
fn reuse_work_current(census: &BoundaryCensus, paired: &PairedCensusEvidence) -> bool {
    let mut cold = Vec::new();
    let mut reused_count = 0;
    for domain in &census.domains {
        if let DomainEvidence::PairedReport { report, reused, .. } = domain.evidence {
            if report >= paired.reports.len() || reused != cold.contains(&report) {
                return false;
            }
            if reused {
                reused_count += 1;
            } else {
                cold.push(report);
            }
        }
    }
    cold.len() == paired.reports.len() && reused_count == paired.reused_reports
}

fn kinded_parents_complete(record: &PairedOccurrenceSnapshot) -> bool {
    let kinded = recorded_kinded_parents(record);
    !record.source.parents.is_empty()
        && !kinded.is_empty()
        && record
            .source
            .parents
            .iter()
            .all(|parent| kinded.contains(&parent))
        && kinded
            .iter()
            .all(|parent| record.source.parents.contains(parent))
}

pub(super) fn restore_operation_obligations(current: &mut Vec<CertificationObligation>) {
    for gap in [
        CertificationObligation::MappedSplitCorrespondence,
        CertificationObligation::CandidateSearchAndCoverage,
        CertificationObligation::SelfIntersection,
        CertificationObligation::Tolerance,
    ] {
        if !current.contains(&gap) {
            current.push(gap);
        }
    }
}

fn recorded_kinded_parents(
    record: &PairedOccurrenceSnapshot,
) -> Vec<&crate::sketch_region::OriginalMemberIdentity> {
    record
        .relations
        .iter()
        .flat_map(|row| {
            row.preserved
                .iter()
                .chain(&row.modified)
                .chain(&row.generated)
        })
        .collect()
}

fn parameter_binding_current(census: &BoundaryCensus, domain: &BoundaryDomain) -> bool {
    let DomainEvidence::PairedReport { source_slots, .. } = domain.evidence else {
        return false;
    };
    Some(source_slots) == super::paired_execution::parameter_slots(census, domain.occurrences)
}
