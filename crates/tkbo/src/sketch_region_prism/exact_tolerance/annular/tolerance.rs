// SPDX-License-Identifier: GPL-2.0-only
//! Later tolerance policy consumes the same bound geometry once; no scalar proof rerun.
use super::{progress::MetadataProgress, *};
use crate::sketch_region_prism::{
    exact_tolerance::{StoredToleranceCap, stored_metadata::refuse_caps, tolerances::*},
    tolerance_inspection::PrismToleranceRequest,
};
/// Reuse exact annular geometry with an explicit metadata cap; `certify_annular_prism_tolerances(join, proof, cap, progress)`.
pub(in crate::sketch_region_prism) fn certify_annular_prism_tolerances(
    join: &AnnularFrozenDomain,
    conformance: &AnnularPrismConformance,
    cap: StoredToleranceCap,
    progress: &mut impl AnnularToleranceProgress,
) -> Result<AnnularPrismTolerance, Box<AnnularToleranceFailure>> {
    let mut work = MetadataWork::new();
    if let Err(cause) = inspect(
        join,
        conformance,
        cap,
        &mut work,
        &mut MetadataProgress(progress),
    ) {
        return Err(Box::new(failure(join, cap, work, cause)));
    }
    Ok(AnnularPrismTolerance {
        binding: join.replay_bytes().to_vec(),
        conformance: conformance.replay_bytes(),
        limits: *join.projection().limits(),
        cap,
        metadata: work.metadata,
    })
}
fn inspect(
    join: &AnnularFrozenDomain,
    conformance: &AnnularPrismConformance,
    cap: StoredToleranceCap,
    work: &mut MetadataWork,
    progress: &mut impl JoinedToleranceProgress,
) -> Result<(), JoinedToleranceCause> {
    work.poll(progress)?;
    if !conformance.binds(join) {
        return Err(JoinedToleranceCause::ConformanceBinding);
    }
    let request = PrismToleranceRequest::new(cap, join.projection().limits().spatial_limit)
        .expect("sealed finite operation limits");
    work.phase = JoinedTolerancePhase::Metadata;
    append_metadata(join.projection(), request, 14, progress, work)?;
    refuse_caps(&work.metadata)?;
    work.phase = JoinedTolerancePhase::Commit;
    work.entity = None;
    work.poll(progress)
}
fn failure(
    join: &AnnularFrozenDomain,
    cap: StoredToleranceCap,
    work: MetadataWork,
    cause: JoinedToleranceCause,
) -> AnnularToleranceFailure {
    AnnularToleranceFailure {
        operation: "annular exact represented-geometry tolerance V2",
        phase: work.phase,
        entity: work.entity,
        cause,
        metadata: work.metadata,
        binding: join.replay_bytes().to_vec(),
        limits: *join.projection().limits(),
        cap,
    }
}
