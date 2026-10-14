// SPDX-License-Identifier: GPL-2.0-only
//! A sealed annular join supplies full domain authority; this helper certifies geometry only.
use super::{progress::*, *};
use crate::sketch_region_prism::exact_tolerance::{
    joined_endpoints::*, joined_frames::*, joined_trims::*,
};
/// Certify immutable actual annular operands; `certify_annular_prism_conformance(join, progress)`.
pub(in crate::sketch_region_prism) fn certify_annular_prism_conformance(
    join: &AnnularFrozenDomain,
    progress: &mut impl AnnularConformanceProgress,
) -> Result<AnnularPrismConformance, Box<AnnularConformanceFailure>> {
    let mut work = AnnularExactWork::new();
    if let Err(cause) = inspect(join, &mut work, &mut GeometryProgress(progress)) {
        return Err(Box::new(work.failure(join, cause)));
    }
    Ok(AnnularPrismConformance {
        binding: join.replay_bytes().to_vec(),
        limits: *join.projection().limits(),
        evidence: work.evidence(),
    })
}
fn inspect(
    join: &AnnularFrozenDomain,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    work.body
        .phase(JoinedConformancePhase::Admission, progress)?;
    count(
        "annular directed raw body vertices",
        join.raw_vertices().len(),
        24,
    )?;
    work.body
        .phase(JoinedConformancePhase::Placements, progress)?;
    certify_projection_frames(join.projection(), &mut work.body, progress)?;
    work.body
        .phase(JoinedConformancePhase::Endpoints, progress)?;
    certify_projection_endpoints(
        join.projection(),
        join.raw_vertices(),
        (12, 24),
        &mut work.body,
        progress,
    )?;
    work.body
        .phase(JoinedConformancePhase::Closures, progress)?;
    super::source_closures::inspect(join.embedding().binding(), work, progress)?;
    work.source_context = None;
    work.body.phase(JoinedConformancePhase::Trims, progress)?;
    certify_projection_trims(
        join.projection(),
        join.faces().iter().map(|face| &face.chart),
        (4, 12),
        &mut work.body,
        progress,
    )?;
    work.body.phase(JoinedConformancePhase::Commit, progress)?;
    Ok(())
}
