// SPDX-License-Identifier: GPL-2.0-only
//! A successful mapped stage and matching observations still cannot publish a healthy body.
use super::{copy_tests::actual_candidate_copy, fixtures::*, transaction::*};
use crate::sketch_region::*;
use crate::sketch_region_candidates::CandidateCause;
use crate::sketch_region_source_conformance::{capture_originals, capture_space_representation};

fn actual_bound_outputs(
    stop: Option<crate::sketch_region_candidates::CandidatePhase>,
) -> PartitionBoundOutputs {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let sources = binding_sources(&edges);
    let originals = capture_originals(&sources).unwrap();
    let observed = observe_partition_space(&sources, &mut |_| false, capture_space_representation);
    let ArrangementCompletion::Arranged(owned) = &observed.completion else {
        panic!("actual rectangle; expected owned mapped arrangement");
    };
    let copied = actual_candidate_copy(owned, stop);
    bind_partition_outputs(observed, Some(copied), originals, &sources)
}
#[test]
fn combined_inspection_retains_real_completion_and_each_unproved_certificate() {
    let bound = actual_bound_outputs(None);
    assert!(matches!(
        bound.arrangement.completion,
        ArrangementCompletion::Arranged(_)
    ));
    assert!(bound.copied.as_ref().unwrap().completion.is_ok());
    let inspection = bound.inspection.as_ref().ok().unwrap();
    assert_eq!(inspection.capture_state, PartitionCaptureState::Captured);
    assert!(inspection.images.iter().all(|row| row.fault.is_none()));
    assert!(inspection.domains.iter().all(|row| row.decision.is_ok()));
    let history = inspection.candidate_history.as_ref().unwrap();
    assert!(history.faults.is_empty());
    assert_required_proofs(&inspection.missing);
}
fn assert_required_proofs(missing: &[RequiredPartitionProof]) {
    assert_eq!(
        missing,
        &[
            RequiredPartitionProof::PairedContactIdentity,
            RequiredPartitionProof::NativeDualParameterTransport,
            RequiredPartitionProof::WorldEndpointBound,
            RequiredPartitionProof::CandidateSearchCoverage,
            RequiredPartitionProof::OrderedBodyValidation,
        ]
    );
}
#[test]
fn cancelled_copy_keeps_original_typed_phase_and_no_successful_candidate_history() {
    let phase = crate::sketch_region_candidates::CandidatePhase::Generation;
    let bound = actual_bound_outputs(Some(phase));
    assert!(matches!(bound.copied.as_ref().unwrap().completion,
        Err(CandidateCause::Cancelled(received)) if received == phase));
    let inspection = bound.inspection.as_ref().ok().unwrap();
    assert!(
        !inspection
            .candidate_history
            .as_ref()
            .unwrap()
            .mapped_candidates_available
    );
    assert_required_proofs(&inspection.missing);
}
