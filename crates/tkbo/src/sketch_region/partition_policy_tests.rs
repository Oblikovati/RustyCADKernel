// SPDX-License-Identifier: GPL-2.0-only
//! An injected source reader cannot run on the default observer=None route.
use super::{arrangement::*, *};
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkbrep::topods::Edge;
struct ForbiddenDefaultSnapshot;
impl ForbiddenDefaultSnapshot {
    fn capture(_edge: &Edge) -> Result<CapturedSpaceRepresentation, CapturedGeometryCause> {
        // A returned error would not prove that the default route skipped this callback.
        panic!("observer=None; expected no source snapshot callback");
    }
}
struct RejectingSpaceSnapshot;
impl RejectingSpaceSnapshot {
    fn capture(_edge: &Edge) -> Result<CapturedSpaceRepresentation, CapturedGeometryCause> {
        Err(CapturedGeometryCause::Unsupported {
            representation: "RejectingSpaceSnapshot",
            dynamic_type: "test-owned refusal".to_owned(),
        })
    }
}
#[test]
fn default_executor_ignores_configured_source_reader_and_preserves_completion() {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let sources = policy_inputs(&edges);
    let completion = arrange_with_executor(
        &sources,
        &mut |_| false,
        &mut MappedArrangementExecutor {
            observation: None,
            space_capture: Some(ForbiddenDefaultSnapshot::capture),
        },
    );
    assert!(matches!(completion, ArrangementCompletion::Arranged(_)));
}
#[test]
fn opted_in_source_capture_failure_is_retained_without_replacing_mapped_result() {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let observed = observe_partition_space(
        &policy_inputs(&edges),
        &mut |_| false,
        RejectingSpaceSnapshot::capture,
    );
    assert!(matches!(
        observed.completion,
        ArrangementCompletion::Arranged(_)
    ));
    let PartitionObservation::Captured(snapshot) = observed.partition else {
        panic!("capture refusal is separate from DS access; expected captured topology metadata");
    };
    assert_eq!(snapshot.sources.len(), 4);
    for source in snapshot.sources {
        assert!(matches!(source.space, Some(Err(_))));
    }
}
fn policy_inputs(shapes: &[rustycad_tkbrep::topods::Shape]) -> Vec<ArrangementSource<'_>> {
    shapes
        .iter()
        .enumerate()
        .map(|(index, shape)| ArrangementSource {
            identity: RegionSourceId(index as u64 + 1),
            shape,
        })
        .collect()
}

#[test]
fn cancel_before_copy_keeps_source_reader_unexecuted() {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let observed = observe_partition_space(
        &policy_inputs(&edges),
        &mut |phase| phase == ArrangementPhase::Copy,
        ForbiddenDefaultSnapshot::capture,
    );
    assert!(matches!(
        observed.completion,
        ArrangementCompletion::Cancelled(ArrangementPhase::Copy)
    ));
    assert!(matches!(
        observed.partition,
        PartitionObservation::NotReached
    ));
}

#[test]
fn cancel_after_interference_preserves_already_captured_topology() {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let observed = observe_partition_space(
        &policy_inputs(&edges),
        &mut |phase| phase == ArrangementPhase::History,
        RejectingSpaceSnapshot::capture,
    );
    assert!(matches!(
        observed.completion,
        ArrangementCompletion::Cancelled(ArrangementPhase::History)
    ));
    assert!(matches!(
        observed.partition,
        PartitionObservation::Captured(_)
    ));
}

#[test]
fn rejected_input_then_reuse_cannot_leak_partition_rows() {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let mut sources = policy_inputs(&edges);
    sources[1].identity = sources[0].identity;
    let failed =
        observe_partition_space(&sources, &mut |_| false, ForbiddenDefaultSnapshot::capture);
    assert!(matches!(
        failed.completion,
        ArrangementCompletion::Declined(ArrangementCause::DuplicateIdentity(RegionSourceId(1)))
    ));
    assert!(matches!(failed.partition, PartitionObservation::NotReached));
    let success = observe_owned_partition(&policy_inputs(&edges), &mut |_| false);
    let PartitionObservation::Captured(snapshot) = success.partition else {
        panic!("independent reuse; expected only current four-source rows");
    };
    assert_eq!(snapshot.sources.len(), 4);
    assert!(snapshot.sources.iter().all(|row| row.space.is_none()));
}
