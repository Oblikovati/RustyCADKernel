// SPDX-License-Identifier: GPL-2.0-only
//! Pure later-stage metadata inspection never reevaluates the geometric certificate.
use super::{
    StoredToleranceCap, joined_types::JoinedPrismConformance, stored_metadata::*,
    tolerance_types::*,
};
use crate::sketch_region_prism::{
    frozen::{proof_join::JoinedPrismProof, *},
    tolerance_inspection::*,
};

/// Reuse exact geometry with an explicit cap; example: `certify_joined_prism_tolerances(join, geometry, cap, progress)`.
pub(crate) fn certify_joined_prism_tolerances(
    join: &JoinedPrismProof,
    conformance: &JoinedPrismConformance,
    cap: StoredToleranceCap,
    progress: &mut impl JoinedToleranceProgress,
) -> Result<JoinedPrismTolerance, Box<JoinedToleranceFailure>> {
    let mut work = MetadataWork::new();
    if let Err(cause) = inspect_metadata(join, conformance, cap, progress, &mut work) {
        return Err(Box::new(work.failure(join, cap, cause)));
    }
    Ok(JoinedPrismTolerance {
        binding: join.replay_bytes().to_vec(),
        conformance: conformance.replay_bytes(),
        operation_limits: *join.projection().limits(),
        cap,
        metadata: work.metadata,
    })
}
fn inspect_metadata(
    join: &JoinedPrismProof,
    conformance: &JoinedPrismConformance,
    cap: StoredToleranceCap,
    progress: &mut impl JoinedToleranceProgress,
    work: &mut MetadataWork,
) -> Result<(), JoinedToleranceCause> {
    work.poll(progress)?;
    if !conformance.binds(join) {
        return Err(JoinedToleranceCause::ConformanceBinding);
    }
    let request = metadata_request(join, cap);
    work.phase = JoinedTolerancePhase::Metadata;
    append_metadata(join.projection(), request, 8, progress, work)?;
    refuse_caps(&work.metadata)?;
    work.phase = JoinedTolerancePhase::Commit;
    work.entity = None;
    work.poll(progress)
}
fn metadata_request(join: &JoinedPrismProof, cap: StoredToleranceCap) -> PrismToleranceRequest {
    // The shared request validates the world quantity but only observes raw metadata here.
    PrismToleranceRequest::new(cap, join.projection().limits().spatial_limit)
        .expect("immutable joined operation already admitted its finite spatial request")
}
pub(super) fn append_metadata(
    projection: &FrozenPrismProjection,
    request: PrismToleranceRequest,
    expected: usize,
    progress: &mut impl JoinedToleranceProgress,
    work: &mut MetadataWork,
) -> Result<(), JoinedToleranceCause> {
    for node in projection
        .nodes()
        .iter()
        .filter(|node| matches!(node.key.kind, 4 | 6 | 7))
    {
        work.entity = Some(node.key);
        work.poll(progress)?;
        work.metadata.push(observe_node(projection, node, request)?);
    }
    complete_metadata_count(work.metadata.len(), expected)
}
fn complete_metadata_count(actual: usize, expected: usize) -> Result<(), JoinedToleranceCause> {
    if actual != expected {
        return Err(JoinedToleranceCause::Census { actual, expected });
    }
    Ok(())
}
pub(super) struct MetadataWork {
    pub phase: JoinedTolerancePhase,
    pub entity: Option<FrozenPrismKey>,
    pub metadata: Vec<JoinedToleranceMetadata>,
}
impl MetadataWork {
    pub fn new() -> Self {
        Self {
            phase: JoinedTolerancePhase::Admission,
            entity: None,
            metadata: Vec::new(),
        }
    }

    pub fn poll(
        &self,
        progress: &mut impl JoinedToleranceProgress,
    ) -> Result<(), JoinedToleranceCause> {
        if progress.cancelled(self.phase) {
            return Err(JoinedToleranceCause::Cancelled);
        }
        Ok(())
    }
    fn failure(
        self,
        join: &JoinedPrismProof,
        cap: StoredToleranceCap,
        cause: JoinedToleranceCause,
    ) -> JoinedToleranceFailure {
        JoinedToleranceFailure {
            operation: "joined exact represented-geometry tolerance V2",
            phase: self.phase,
            entity: self.entity,
            cause,
            metadata: self.metadata,
            binding: join.replay_bytes().to_vec(),
            operation_limits: *join.projection().limits(),
            cap,
        }
    }
}
