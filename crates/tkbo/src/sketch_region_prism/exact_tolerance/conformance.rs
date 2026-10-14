// SPDX-License-Identifier: GPL-2.0-only
//! This immutable geometry proof is reusable before the ordered tolerance stage.
use super::{joined_progress::*, joined_types::*};
use crate::sketch_region_prism::frozen::proof_join::JoinedPrismProof;

/// Certify exact raw geometry; example: `certify_joined_prism_conformance(join, progress)`.
pub(crate) fn certify_joined_prism_conformance(
    join: &JoinedPrismProof,
    progress: &mut impl JoinedConformanceProgress,
) -> Result<JoinedPrismConformance, JoinedConformanceFailure> {
    let mut work = ConformanceWork::new();
    if let Err(cause) = run_conformance(join, &mut work, progress) {
        return Err(work.failed(cause, join));
    }
    Ok(JoinedPrismConformance {
        binding: join.replay_bytes().to_vec(),
        operation_limits: *join.projection().limits(),
        evidence: work.evidence,
    })
}

fn run_conformance(
    join: &JoinedPrismProof,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    work.phase(JoinedConformancePhase::Admission, progress)?;
    count(
        "raw directed body vertex contexts",
        join.raw_vertices().len(),
        12,
    )?;
    work.phase(JoinedConformancePhase::Placements, progress)?;
    super::joined_frames::certify_projection_frames(join.projection(), work, progress)?;
    work.phase(JoinedConformancePhase::Endpoints, progress)?;
    super::joined_endpoints::certify_body_endpoints(join, work, progress)?;
    work.phase(JoinedConformancePhase::Closures, progress)?;
    super::joined_closures::certify_source_closures(join, work, progress)?;
    work.phase(JoinedConformancePhase::Trims, progress)?;
    super::joined_trims::certify_joined_trims(join, work, progress)?;
    work.phase(JoinedConformancePhase::Commit, progress)
}
