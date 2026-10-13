// SPDX-License-Identifier: GPL-2.0-only
//! Represented parents are necessary coverage evidence, not an exhaustive region-search proof.
use super::outcome::*;
use crate::sketch_region_candidates::CandidateBatch;

/// Retain represented/unrepresented input occurrences; `inspect_candidate_coverage(&census,&batch)`.
/// Shared parents bind copied topology; no parameter partition is inferred from those keys alone.
pub(crate) fn inspect_candidate_coverage(
    census: &BoundaryCensus,
    candidates: &CandidateBatch,
) -> Vec<CandidateOccurrenceCoverage> {
    census
        .inputs
        .segments
        .iter()
        .enumerate()
        .map(|(input, source)| CandidateOccurrenceCoverage {
            input,
            represented_occurrences: represented_sources(source, candidates),
            parameter_coverage_certified: false,
        })
        .collect()
}
fn represented_sources(
    source: &crate::sketch_region_candidates::CandidateSegment,
    candidates: &CandidateBatch,
) -> Vec<u64> {
    candidates
        .candidates
        .iter()
        .flat_map(|candidate| &candidate.wires)
        .flat_map(|wire| &wire.segments)
        .filter(|segment| {
            source
                .parents
                .iter()
                .all(|parent| segment.parents.contains(parent))
        })
        .map(|segment| segment.occurrence)
        .collect()
}
