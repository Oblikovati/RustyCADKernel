// SPDX-License-Identifier: GPL-2.0-only
//! Preserve original arrangement/copy outcomes; observers do not turn partial work into success.
use super::{classification::*, diagnostics::*, pipeline::enter, progress::*};
use crate::sketch_region::{
    ArrangementCompletion, ArrangementPhase, ArrangementSource, ObservedPartitionArrangement,
    PartitionOccurrenceIdentity, observe_partition_space,
};
use crate::sketch_region_candidates::{CandidateCopyInspection, observe_candidate_copies};
use crate::sketch_region_partition_bindings::bind_partition_outputs;
use crate::sketch_region_source_conformance::{capture_originals, capture_space_representation};
pub(super) fn partition(
    retained: &mut PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Result<(), OwnedPreparationCause> {
    let phase = OwnedPrismPhase::Partition(ArrangementPhase::Admission);
    enter(retained, callbacks, phase)?;
    let sources = [retained.source()?.source()];
    let originals = capture_originals(&sources).map_err(OwnedPreparationCause::Native)?;
    let observed = observe(&sources, callbacks);
    let copied = candidates(&observed, retained.request.declared_plane, callbacks);
    retained.partition = Some(bind_partition_outputs(
        observed, copied, originals, &sources,
    ));
    require_candidates(retained)
}
fn observe(
    sources: &[ArrangementSource<'_>],
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> ObservedPartitionArrangement {
    observe_partition_space(
        sources,
        &mut |p| callbacks.poll(OwnedPrismPhase::Partition(p)),
        capture_space_representation,
    )
}
fn candidates(
    observed: &ObservedPartitionArrangement,
    plane: rustycad_tkmath::gp::Plane3,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Option<CandidateCopyInspection> {
    let ArrangementCompletion::Arranged(owned) = &observed.completion else {
        return None;
    };
    Some(observe_candidate_copies(
        owned,
        plane,
        &mut |p| callbacks.poll(OwnedPrismPhase::Candidate(p)),
        PartitionOccurrenceIdentity::capture,
    ))
}
fn require_candidates(retained: &PreparationRetained) -> Result<(), OwnedPreparationCause> {
    let partition = local(retained.partition.as_ref(), "bound partition")?;
    if !matches!(
        partition.arrangement.completion,
        ArrangementCompletion::Arranged(_)
    ) {
        return Err(OwnedPreparationCause::StageUnavailable {
            class: arrangement_class(&partition.arrangement.completion),
        });
    }
    let copied = local(
        partition.copied.as_ref(),
        "actual candidate copy observation",
    )?;
    copied.completion.as_ref().map(|_| ()).map_err(|cause| {
        OwnedPreparationCause::StageUnavailable {
            class: candidate_class(cause),
        }
    })
}
