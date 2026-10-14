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
}
pub(crate) struct CandidateImageCopyLink {
    pub original: PartitionOccurrenceIdentity,
    pub copied: PartitionOccurrenceIdentity,
    pub preserved: Vec<OriginalMemberIdentity>,
    pub modified: Vec<OriginalMemberIdentity>,
    pub generated: Vec<OriginalMemberIdentity>,
}
pub(super) struct CandidateCopyCollector {
    links: Vec<CandidateImageCopyLink>,
    capture: fn(&Shape) -> PartitionOccurrenceIdentity,
}
/// Keep actual copy links alongside the unchanged candidate result.
/// Example: `observe_candidate_copies(&owned, plane, &mut |_|false, capture_occurrence)`.
pub(crate) fn observe_candidate_copies(
    owned: &OwnedArrangement,
    plane: Plane3,
    cancelled: &mut dyn FnMut(CandidatePhase) -> bool,
    capture: fn(&Shape) -> PartitionOccurrenceIdentity,
) -> CandidateCopyInspection {
    let mut collector = CandidateCopyCollector {
        links: Vec::new(),
        capture,
    };
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
    collector.links.push(CandidateImageCopyLink {
        original: (collector.capture)(&original.output),
        copied: (collector.capture)(copied),
        preserved: original.preserved.clone(),
        modified: original.modified.clone(),
        generated: original.generated.clone(),
    });
}

impl CandidateCopyCollector {
    fn finish(self, completion: Result<CandidateBatch, CandidateCause>) -> CandidateCopyInspection {
        CandidateCopyInspection {
            completion,
            links: self.links,
        }
    }
}
