// SPDX-License-Identifier: GPL-2.0-only
//! IDs locate the producer but complete descriptor/physical binding is required on reuse.
use super::{BoundaryCensus, DomainEvidence};
use crate::sketch_region_candidates::CandidateSegment;
use rustycad_tkmath::predicates::roots::bezier_line::*;
pub(super) fn report_pair_bound(census: &BoundaryCensus, report: usize, pair: [usize; 2]) -> bool {
    let Some(report) = census.reports.get(report) else {
        return false;
    };
    let source = report.source();
    let (Some(curve), Some(line)) = (
        census.inputs.segments.get(pair[0]),
        census.inputs.segments.get(pair[1]),
    ) else {
        return false;
    };
    binds_curve(source, curve)
        && binds_line(source, line)
        && producer_bound(census, source, curve, line)
        && source.domain() == BezierLineDomain::Segment
        && source.domain_sign() == BezierDomainSignOptions::source_basis()
        && same_policy(source.root_options(), census.requested.roots)
        && source.requested_world_error().to_bits() == census.requested.max_world_error.to_bits()
}
fn binds_curve(source: &BezierIncidenceSource, curve: &CandidateSegment) -> bool {
    equal_bits(
        source.controls().iter().flatten(),
        curve.controls.iter().flatten(),
    ) && source.weights().is_some() == curve.weights.is_some()
        && equal_bits(
            source.weights().iter().flat_map(|w| w.iter()),
            curve.weights.iter().flatten(),
        )
        && equal_bits(source.interval().iter(), curve.interval.iter())
}
fn binds_line(source: &BezierIncidenceSource, line: &CandidateSegment) -> bool {
    line.controls.len() == 2
        && line.weights.is_none()
        && line.interval == [0., 1.]
        && equal_bits(
            source.line().iter().flatten(),
            line.controls.iter().flatten(),
        )
}
fn producer_bound(
    census: &BoundaryCensus,
    source: &BezierIncidenceSource,
    curve: &CandidateSegment,
    line: &CandidateSegment,
) -> bool {
    let producer_curve = unique_producer(census, source.curve_id());
    let producer_line = unique_producer(census, source.line_id());
    matches!((producer_curve,producer_line),(Some(a),Some(b)) if a.edge.is_same(&curve.edge) && b.edge.is_same(&line.edge) && binds_curve(source,a) && binds_line(source,b))
}
fn same_policy(
    a: rustycad_tkmath::predicates::roots::RootIsolationOptions,
    b: rustycad_tkmath::predicates::roots::RootIsolationOptions,
) -> bool {
    a.version == b.version
        && a.max_degree == b.max_degree
        && a.max_bisections == b.max_bisections
        && a.max_width.to_bits() == b.max_width.to_bits()
}
fn equal_bits<'a>(a: impl Iterator<Item = &'a f64>, b: impl Iterator<Item = &'a f64>) -> bool {
    a.map(|v| v.to_bits()).eq(b.map(|v| v.to_bits()))
}
pub(super) fn report_domain_pair(census: &BoundaryCensus, target: usize) -> Option<[usize; 2]> {
    census
        .domains
        .iter()
        .find_map(|domain| match domain.evidence {
            DomainEvidence::StoredIncidence {
                report,
                curve,
                line,
            } if report == target => Some([curve, line]),
            _ => None,
        })
}

fn unique_producer(census: &BoundaryCensus, occurrence: u64) -> Option<&CandidateSegment> {
    let mut matches = census
        .inputs
        .segments
        .iter()
        .filter(|s| s.occurrence == occurrence);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}
