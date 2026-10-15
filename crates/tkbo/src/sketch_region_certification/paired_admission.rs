// SPDX-License-Identifier: GPL-2.0-only
//! The actual source-bound boundary selects paired evidence; the legacy one-axis entry remains diagnostic only.
use super::{census, outcome::*, paired_execution, paired_types::*};
use crate::sketch_region_source_conformance::SourceConformancePolicy;
use rustycad_tkmath::predicates::roots::bezier_pair::*;

/// Admit one general pair/self census; `enumerate_source_bound_census(inputs,policy,source_policy,||false)`.
/// Source metric work has its own explicit policy; numerical pair work shares one operation-wide budget.
pub(crate) fn enumerate_source_bound_census(
    inputs: BoundaryInputs,
    policy: CensusPolicy,
    source_policy: SourceConformancePolicy,
    cancelled: impl FnMut() -> bool,
) -> Result<BoundaryCensus, CensusCause> {
    enumerate_with_pair_version(inputs, (policy, source_policy), 1, cancelled)
}
/// ADR 0066 part B: the same census with pair policy version 2, whose distinct pairs on one
/// proper conic or line complete by the exact common-carrier route.
/// Example: `enumerate_carrier_census(inputs, policy, source_policy, || false)`.
pub(crate) fn enumerate_carrier_census(
    inputs: BoundaryInputs,
    policy: CensusPolicy,
    source_policy: SourceConformancePolicy,
    cancelled: impl FnMut() -> bool,
) -> Result<BoundaryCensus, CensusCause> {
    enumerate_with_pair_version(inputs, (policy, source_policy), 2, cancelled)
}
fn enumerate_with_pair_version(
    inputs: BoundaryInputs,
    policies: (CensusPolicy, SourceConformancePolicy),
    pair_version: u32,
    mut cancelled: impl FnMut() -> bool,
) -> Result<BoundaryCensus, CensusCause> {
    let (policy, source_policy) = policies;
    census::validate_policy(policy).map_err(CensusCause::Policy)?;
    let mut retained = census::empty_census(inputs, policy);
    let mut evidence = capture_pair_evidence(&retained);
    evidence.requested.version = pair_version;
    retained.paired = Some(evidence);
    if admit_source_binding(&mut retained, source_policy, &mut cancelled) {
        return Err(CensusCause::SourceCancelled {
            completed: Box::new(retained),
        });
    }
    paired_execution::enumerate_paired_domains(retained, &mut cancelled)
}
pub(super) fn admit_source_binding(
    retained: &mut BoundaryCensus,
    source_policy: SourceConformancePolicy,
    cancelled: &mut impl FnMut() -> bool,
) -> bool {
    retained.source_policy = Some(source_policy);
    let stopped = census::retain_source_binding(retained, source_policy, cancelled);
    if super::source_status::representation_matches(retained) {
        retained
            .missing
            .retain(|gap| *gap != CertificationObligation::SourceConversionAndPlane);
    }
    stopped
}
fn capture_pair_evidence(census: &BoundaryCensus) -> PairedCensusEvidence {
    PairedCensusEvidence {
        requested: BezierPairPolicy {
            version: 1,
            roots: census.requested.roots,
            signs: BezierPairSignOptions::derived(),
            requested_world_error: census.requested.max_world_error,
        },
        occurrences: captured_occurrences(census),
        reports: Vec::new(),
        reused_reports: 0,
    }
}
fn captured_occurrences(census: &BoundaryCensus) -> Vec<PairedOccurrenceSnapshot> {
    census
        .inputs
        .segments
        .iter()
        .enumerate()
        .map(|(index, source)| captured_occurrence(census, index, source))
        .collect()
}
fn captured_occurrence(
    census: &BoundaryCensus,
    index: usize,
    source: &crate::sketch_region_candidates::CandidateSegment,
) -> PairedOccurrenceSnapshot {
    PairedOccurrenceSnapshot {
        source: source.clone(),
        // WHY: aliases share actual source parents; retain every located row's full key and kind unchanged.
        relations: source_history_rows(census, source),
        physical_source: census.inputs.segments[..=index]
            .iter()
            .position(|earlier| census::same_capture(earlier, source))
            .unwrap_or(index),
    }
}

fn source_history_rows(
    census: &BoundaryCensus,
    source: &crate::sketch_region_candidates::CandidateSegment,
) -> Vec<crate::sketch_region::ArrangementRelation> {
    census
        .inputs
        .relations
        .iter()
        .filter(|row| row.output.is_same(&source.edge))
        .cloned()
        .collect()
}
