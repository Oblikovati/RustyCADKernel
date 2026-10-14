// SPDX-License-Identifier: GPL-2.0-only
//! The callback uses the checked actual occurrence helper, never synthetic copy keys.
use super::{fixtures::*, image_links::*};
use crate::sketch_region::*;
use crate::sketch_region_candidates::*;
use rustycad_tkmath::gp::Plane3;

pub(super) fn actual_copy_inputs() -> (OwnedPartitionSnapshot, OwnedArrangement) {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let observed = observe_owned_partition(&binding_sources(&edges), &mut |_| false);
    let PartitionObservation::Captured(snapshot) = observed.partition else {
        panic!("rectangle source; expected captured source/image context");
    };
    let ArrangementCompletion::Arranged(owned) = observed.completion else {
        panic!("rectangle source; expected mapped owned arrangement");
    };
    (snapshot, owned)
}
pub(super) fn actual_candidate_copy(
    owned: &OwnedArrangement,
    stop: Option<CandidatePhase>,
) -> CandidateCopyInspection {
    observe_candidate_copies(
        owned,
        Plane3::default(),
        &mut |phase| Some(phase) == stop,
        PartitionOccurrenceIdentity::capture,
    )
}
#[test]
fn actual_modified_links_bind_every_unsplit_source_image_and_all_parents() {
    let (snapshot, owned) = actual_copy_inputs();
    let copied = actual_candidate_copy(&owned, None);
    assert!(copied.completion.is_ok());
    let bindings = inspect_image_links(&snapshot, &copied.links);
    assert_eq!(bindings.len(), 4);
    assert!(bindings.iter().all(|binding| binding.fault.is_none()));
    assert!(
        copied
            .links
            .iter()
            .all(|row| row.original.topology_address != row.copied.topology_address)
    );
}
#[test]
fn copy_boundary_cancel_keeps_actual_modified_reader_unexecuted() {
    let (_, owned) = actual_copy_inputs();
    let copied = actual_candidate_copy(&owned, Some(CandidatePhase::Copy));
    assert!(matches!(
        copied.completion,
        Err(CandidateCause::Cancelled(CandidatePhase::Copy))
    ));
    assert!(copied.links.is_empty());
}
#[test]
fn generation_cancel_retains_actual_copy_rows_without_successful_candidates() {
    let (snapshot, owned) = actual_copy_inputs();
    let copied = actual_candidate_copy(&owned, Some(CandidatePhase::Generation));
    assert!(matches!(
        copied.completion,
        Err(CandidateCause::Cancelled(CandidatePhase::Generation))
    ));
    assert!(
        inspect_image_links(&snapshot, &copied.links)
            .iter()
            .all(|binding| binding.fault.is_none())
    );
}
#[test]
fn cancelled_then_successful_copy_reuses_inputs_without_accumulating_receipt_rows() {
    let (_, owned) = actual_copy_inputs();
    let cancelled = actual_candidate_copy(&owned, Some(CandidatePhase::Generation));
    let success = actual_candidate_copy(&owned, None);
    assert!(success.completion.is_ok());
    assert_eq!(cancelled.links.len(), success.links.len());
    for (before, after) in cancelled.links.iter().zip(&success.links) {
        assert_eq!(before.original, after.original);
        // Copy keys are operation-local: released allocations may reuse addresses.
        assert_eq!(before.modified, after.modified);
    }
}

#[test]
fn actual_null_copy_failure_then_success_keeps_native_cause_and_no_stale_links() {
    let (_, mut owned) = actual_copy_inputs();
    let valid = std::mem::take(&mut owned.arrangement);
    let failed = actual_candidate_copy(&owned, None);
    let Err(CandidateCause::Mapped(cause)) = &failed.completion else {
        panic!("actual null copy; expected original mapped Failure");
    };
    let cause_before = (cause.exception_type(), cause.message().to_owned());
    assert_failed_copy_observation(&failed);
    owned.arrangement = valid;
    assert!(actual_candidate_copy(&owned, None).completion.is_ok());
    assert_eq!(
        cause_before,
        (cause.exception_type(), cause.message().to_owned())
    );
}

fn assert_failed_copy_observation(failed: &CandidateCopyInspection) {
    assert!(failed.links.is_empty());
    assert!(matches!(
        failed.generated_topology,
        GeneratedTopologyObservation::NotReached
    ));
    assert!(failed.observation_failures.is_empty());
}
