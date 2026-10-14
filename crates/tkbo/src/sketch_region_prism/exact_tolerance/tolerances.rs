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
) -> Result<JoinedPrismTolerance, JoinedToleranceFailure> {
    let mut work = MetadataWork {
        phase: JoinedTolerancePhase::Admission,
        entity: None,
        metadata: Vec::new(),
    };
    if let Err(cause) = inspect_metadata(join, conformance, cap, progress, &mut work) {
        return Err(work.failure(join, cap, cause));
    }
    Ok(JoinedPrismTolerance {
        binding: join.replay_bytes().to_vec(),
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
    // The shared request validates the world quantity but only observes raw metadata here.
    let request = PrismToleranceRequest::new(cap, join.projection().limits().spatial_limit)
        .expect("immutable joined operation already admitted its finite spatial request");
    work.phase = JoinedTolerancePhase::Metadata;
    append_metadata(join.projection(), request, progress, work)?;
    refuse_caps(&work.metadata)?;
    work.phase = JoinedTolerancePhase::Commit;
    work.entity = None;
    work.poll(progress)
}
fn append_metadata(
    projection: &FrozenPrismProjection,
    request: PrismToleranceRequest,
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
    if work.metadata.len() != 8 {
        return Err(JoinedToleranceCause::Census {
            actual: work.metadata.len(),
            expected: 8,
        });
    }
    Ok(())
}
struct MetadataWork {
    phase: JoinedTolerancePhase,
    entity: Option<FrozenPrismKey>,
    metadata: Vec<JoinedToleranceMetadata>,
}
impl MetadataWork {
    fn poll(
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
