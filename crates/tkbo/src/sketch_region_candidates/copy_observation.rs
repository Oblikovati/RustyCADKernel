// SPDX-License-Identifier: GPL-2.0-only
//! Bind copies at the actual mapped Modified call; ancestry never reconstructs copy identity.
use super::outcome::*;
use crate::sketch_region::{
    ArrangementRelation, OriginalMemberIdentity, OwnedArrangement, PartitionOccurrenceIdentity,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Plane3;

pub(crate) struct CandidateCopyInspection {
    pub completion: Result<CandidateBatch, CandidateCause>,
    pub links: Vec<CandidateImageCopyLink>,
    pub generated_topology: super::generated_observation::GeneratedTopologyObservation,
    pub observation_failures: Vec<CandidateCause>,
    pub copied_tree: super::copy_tree::CandidateCopyTreeObservation,
}
#[derive(Clone)]
pub(crate) struct CandidateImageCopyLink {
    pub original: PartitionOccurrenceIdentity,
    pub kind: Result<
        rustycad_tkg3d::top_abs::ShapeEnum,
        std::sync::Arc<rustycad_tkernel::standard::Failure>,
    >,
    pub history_supported: Result<bool, std::sync::Arc<rustycad_tkernel::standard::Failure>>,
    pub copied: PartitionOccurrenceIdentity,
    pub preserved: Vec<OriginalMemberIdentity>,
    pub modified: Vec<OriginalMemberIdentity>,
    pub generated: Vec<OriginalMemberIdentity>,
}
pub(super) struct CandidateCopyCollector {
    links: Vec<CandidateImageCopyLink>,
    capture: fn(&Shape) -> PartitionOccurrenceIdentity,
    copied_tree: super::copy_tree::CandidateCopyTreeObservation,
    generated_topology: super::generated_observation::GeneratedTopologyObservation,
    observation_failures: Vec<CandidateCause>,
}
/// Keep actual copy links alongside the unchanged candidate result.
/// Example: `observe_candidate_copies(&owned, plane, &mut |_|false, capture_occurrence)`.
pub(crate) fn observe_candidate_copies(
    owned: &OwnedArrangement,
    plane: Plane3,
    cancelled: &mut dyn FnMut(CandidatePhase) -> bool,
    capture: fn(&Shape) -> PartitionOccurrenceIdentity,
) -> CandidateCopyInspection {
    let mut collector = CandidateCopyCollector::new(capture);
    let completion = super::generation::extract_candidates_observing_copy(
        owned,
        plane,
        cancelled,
        Some(&mut collector),
    );
    collector.finish(completion)
}
pub(super) fn record_copy_if_present(
    collector: Option<&mut CandidateCopyCollector>,
    original: &ArrangementRelation,
    copied: &Shape,
) {
    let Some(collector) = collector else {
        return;
    };
    collector
        .links
        .push(snapshot_copy_link(original, copied, collector.capture));
}
fn snapshot_copy_link(
    original: &ArrangementRelation,
    copied: &Shape,
    capture: fn(&Shape) -> PartitionOccurrenceIdentity,
) -> CandidateImageCopyLink {
    use rustycad_tkbrep::brep_tools::History;
    CandidateImageCopyLink {
        original: capture(&original.output),
        kind: original.output.shape_type().map_err(std::sync::Arc::new),
        history_supported: History::is_supported_type(&original.output)
            .map_err(std::sync::Arc::new),
        copied: capture(copied),
        preserved: original.preserved.clone(),
        modified: original.modified.clone(),
        generated: original.generated.clone(),
    }
}

impl CandidateCopyCollector {
    fn new(capture: fn(&Shape) -> PartitionOccurrenceIdentity) -> Self {
        Self {
            links: Vec::new(),
            capture,
            generated_topology:
                super::generated_observation::GeneratedTopologyObservation::NotReached,
            observation_failures: Vec::new(),
            copied_tree: super::copy_tree::CandidateCopyTreeObservation::NotReached,
        }
    }

    fn finish(self, completion: Result<CandidateBatch, CandidateCause>) -> CandidateCopyInspection {
        CandidateCopyInspection {
            completion,
            links: self.links,
            copied_tree: self.copied_tree,
            generated_topology: self.generated_topology,
            observation_failures: self.observation_failures,
        }
    }
}

pub(super) fn capture_tree_if_present(
    collector: Option<&mut CandidateCopyCollector>,
    copier: &mut rustycad_tktopalgo::brep_builder_api::BRepBuilderAPICopy,
) {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
    let Some(collector) = collector else {
        return;
    };
    // An observer read failure must not replace the mapped phase/callback result.
    collector.copied_tree = match copier.shape() {
        Ok(shape) => super::copy_tree::snapshot_copied_tree(shape),
        Err(cause) => super::copy_tree::CandidateCopyTreeObservation::Incomplete {
            members: Vec::new(),
            cause: std::sync::Arc::new(cause),
        },
    };
}

pub(super) fn capture_generated_if_present(
    collector: Option<&mut CandidateCopyCollector>,
    builder: &crate::bop_algo::BuilderFace,
    parents: &super::generation::CandidateParents,
) {
    let Some(collector) = collector else {
        return;
    };
    collector.generated_topology = super::generated_observation::snapshot_generated_areas(
        builder,
        parents,
        &collector.copied_tree,
        &mut collector.observation_failures,
    );
}
