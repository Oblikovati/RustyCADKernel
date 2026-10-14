// SPDX-License-Identifier: GPL-2.0-only
use super::{copy_tests::actual_candidate_copy, corpus::*, fixtures::*, transaction::*};
use crate::sketch_region::*;
use crate::sketch_region_candidates::{CandidateCause, CandidateGap};
use crate::sketch_region_source_conformance::{capture_originals, capture_space_representation};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(crate) fn inspect_corpus_sources(sources: &[Shape]) -> PartitionBoundOutputs {
    let inputs = binding_sources(sources);
    let originals = capture_originals(&inputs).unwrap();
    let observed = observe_partition_space(&inputs, &mut |_| false, capture_space_representation);
    let ArrangementCompletion::Arranged(owned) = &observed.completion else {
        panic!(
            "source count={}; expected mapped arrangement",
            sources.len()
        );
    };
    let copied = actual_candidate_copy(owned, None);
    bind_partition_outputs(observed, Some(copied), originals, &inputs)
}
#[test]
fn split_unsplit_orientation_location_distinct_topology_and_aliases_use_one_consumer() {
    for mode in 0..7 {
        let sources = partition_binding_sources(mode);
        let inspected = inspect_corpus_sources(&sources);
        assert_corpus_correspondence(mode, &inspected);
    }
}
fn assert_corpus_correspondence(mode: usize, inspected: &PartitionBoundOutputs) {
    assert_mapped_candidate_stage(mode, inspected);
    let receipt = inspected.inspection.as_ref().ok().unwrap();
    assert_partition_correspondence(mode, receipt);
    assert!(
        receipt.images.iter().all(|row| row.fault.is_none()),
        "mode={mode}; unbound source image"
    );
    assert!(
        receipt.domains.iter().all(|row| row.decision.is_ok()),
        "mode={mode}; unbound source descriptor"
    );
    assert_stage_correspondence(mode, receipt);
}
fn assert_stage_correspondence(mode: usize, receipt: &PartitionTransactionInspection) {
    let generated = receipt.generated_bindings.as_ref().unwrap();
    assert!(generated.stage_observed);
    assert!(
        generated.faults.is_empty(),
        "mode={mode}; generated history faults"
    );
    let history = receipt.candidate_history.as_ref().unwrap();
    assert_eq!(history.mapped_candidates_available, mode > 1);
    assert!(
        history.faults.is_empty(),
        "mode={mode}; candidate history mismatch"
    );
}
fn assert_partition_correspondence(mode: usize, receipt: &PartitionTransactionInspection) {
    assert_eq!(receipt.capture_state, PartitionCaptureState::Captured);
    let endpoints = receipt.endpoints.as_ref().unwrap();
    assert!(
        endpoints.structural_faults.is_empty(),
        "mode={mode}; unexpected DS faults"
    );
    assert!(
        endpoints.image_span_faults.is_empty(),
        "mode={mode}; unexpected representative images"
    );
    assert!(
        endpoints
            .native_spans
            .iter()
            .all(|row| row.observation.is_ok()),
        "mode={mode}; noncontiguous native spans"
    );
}

#[test]
fn unsupported_real_circle_capture_then_success_reuse_preserves_typed_failure_and_inputs() {
    let circle = circle_source();
    let before = super::isolation_tests::circle_input_stamp(&circle);
    let failed = inspect_corpus_sources(std::slice::from_ref(&circle));
    assert!(matches!(
        &failed.copied.as_ref().unwrap().completion,
        Err(CandidateCause::Incomplete(
            CandidateGap::UnsupportedCurve { .. }
        ))
    ));

    let success = inspect_corpus_sources(&partition_binding_sources(2));
    assert_corpus_correspondence(2, &success);
    assert_eq!(before, super::isolation_tests::circle_input_stamp(&circle));
}

fn assert_mapped_candidate_stage(mode: usize, inspected: &PartitionBoundOutputs) {
    let copied = inspected.copied.as_ref().unwrap();
    if mode > 1 {
        assert!(
            copied.completion.is_ok(),
            "mode={mode}; expected captured candidates"
        );
        return;
    }
    assert!(matches!(&copied.completion,
        Err(CandidateCause::Incomplete(CandidateGap::IncompleteWire { omitted, unexpected }))
        if omitted.len() == (if mode == 0 { 2 } else { 4 }) && unexpected.is_empty()));
    assert_generated_internal_stage(mode, copied);
}
fn assert_generated_internal_stage(
    mode: usize,
    copied: &crate::sketch_region_candidates::CandidateCopyInspection,
) {
    use crate::sketch_region_candidates::GeneratedTopologyObservation;
    let GeneratedTopologyObservation::Observed(areas) = &copied.generated_topology else {
        panic!("expected independently retained generated-area observation");
    };
    assert_eq!(areas.len(), if mode == 0 { 2 } else { 1 });
    let internal: Vec<_> = areas
        .iter()
        .flat_map(|area| &area.members)
        .filter(|row| {
            row.kind == ShapeEnum::Edge && row.occurrence.orientation == Orientation::Internal
        })
        .collect();
    assert_eq!(internal.len(), if mode == 0 { 3 } else { 4 });
    assert_internal_members(&internal);
    assert!(copied.observation_failures.is_empty());
}
fn assert_internal_members(members: &[&crate::sketch_region_candidates::GeneratedMemberSnapshot]) {
    assert!(!members.is_empty());
    assert!(members.iter().all(|row| row.copy_indices.len() == 1));
    assert!(
        members
            .iter()
            .all(|row| !row.modified.is_empty() || !row.generated.is_empty())
    );
}

#[test]
fn unchanged_default_capture_preserves_internal_wire_failure_without_observation() {
    let sources = partition_binding_sources(0);
    let observed = observe_owned_partition(&binding_sources(&sources), &mut |_| false);
    let ArrangementCompletion::Arranged(owned) = observed.completion else {
        panic!("quadratic sources; expected original arrangement");
    };
    let direct = crate::sketch_region_candidates::extract_candidates(
        &owned,
        rustycad_tkmath::gp::Plane3::default(),
        &mut |_| false,
    );
    assert!(matches!(direct,
        Err(CandidateCause::Incomplete(CandidateGap::IncompleteWire { omitted, unexpected }))
        if omitted.len() == 2 && unexpected.is_empty()));
}
#[test]
fn capture_phase_cancel_retains_stage_observation_without_admitting_candidates() {
    let (_, owned) = super::copy_tests::actual_copy_inputs();
    let observed = super::copy_tests::actual_candidate_copy(
        &owned,
        Some(crate::sketch_region_candidates::CandidatePhase::Capture),
    );
    assert!(matches!(
        observed.completion,
        Err(CandidateCause::Cancelled(
            crate::sketch_region_candidates::CandidatePhase::Capture
        ))
    ));
    assert!(matches!(
        observed.generated_topology,
        crate::sketch_region_candidates::GeneratedTopologyObservation::Observed(_)
    ));
    assert!(observed.observation_failures.is_empty());
}
