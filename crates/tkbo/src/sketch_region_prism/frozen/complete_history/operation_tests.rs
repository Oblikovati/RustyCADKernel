// SPDX-License-Identifier: GPL-2.0-only
//! The operation gate owns its operands and never invokes user code after both final rechecks.
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use fixture::HistoryFixture;
fn inspected(result: PrismHistoryOutcome<'_>) -> Box<PrismCompleteHistory> {
    match result {
        PrismHistoryOutcome::Inspected(value) => value,
        PrismHistoryOutcome::Incomplete(f) | PrismHistoryOutcome::Failed(f) => {
            panic!("{:?}", f.finding)
        }
        PrismHistoryOutcome::Cancelled { phase, .. } => panic!("unexpected cancellation {phase:?}"),
    }
}
pub(super) fn fixture_bytes() -> Vec<u8> {
    let fixture = HistoryFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    inspected(inspect_complete_prism_history(
        &fixture.request(&sources),
        |_| false,
    ))
    .replay_bytes()
    .to_vec()
}
#[test]
fn every_history_phase_cancellation_returns_no_partial_evidence_and_preserves_reuse() {
    let fixture = HistoryFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let baseline = inspected(inspect_complete_prism_history(&request, |_| false));
    for stop in [
        PrismHistoryPhase::Admission,
        PrismHistoryPhase::Capture,
        PrismHistoryPhase::Commit,
    ] {
        let mut reached = false;
        let result = inspect_complete_prism_history(&request, |phase| {
            reached |= phase == stop;
            phase == stop
        });
        assert!(reached);
        assert!(matches!(result, PrismHistoryOutcome::Cancelled { phase, .. } if phase == stop));
        assert_eq!(
            inspected(inspect_complete_prism_history(&request, |_| false)).replay_bytes(),
            baseline.replay_bytes()
        );
    }
}
#[test]
fn history_commit_rebind_detects_partition_root_change_without_changing_any_geometry_scalar() {
    let fixture = HistoryFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    let request = fixture.request(&sources);
    let baseline = inspected(inspect_complete_prism_history(&request, |_| false));
    let bytes = baseline.replay_bytes().to_vec();
    let mut root = relations::arranged(&request).unwrap().arrangement.clone();
    let child = rustycad_tkbrep::topods::ShapeIterator::new(&root, true, true)
        .value()
        .unwrap()
        .clone();
    let mut changed = false;
    let result = inspect_complete_prism_history(&request, |phase| {
        if phase == PrismHistoryPhase::Commit {
            root.set_free(true).unwrap();
            rustycad_tkbrep::topods::Builder
                .remove(&mut root, &child)
                .unwrap();
            changed = true;
        }
        false
    });
    assert!(changed);
    assert!(matches!(
        result,
        PrismHistoryOutcome::Incomplete(_) | PrismHistoryOutcome::Failed(_)
    ));
    assert_eq!(baseline.replay_bytes(), bytes);
}
#[test]
fn history_only_reproject_detects_rejected_history_changes_outside_joined_body() {
    let mut fixture = HistoryFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    let baseline = inspected(inspect_complete_prism_history(
        &fixture.request(&sources),
        |_| false,
    ));
    let bytes = baseline.replay_bytes().to_vec();
    fixture.operation.receipt.input.rejected[0][0]
        .generated
        .pop();
    assert!(reproject_complete_prism_history(&fixture.request(&sources), &baseline).is_err());
    assert_eq!(baseline.replay_bytes(), bytes);
}
#[test]
fn wrong_operation_extra_authorship_and_invalid_bounds_refuse_before_acceptance() {
    let mut fixture = HistoryFixture::new();
    fixture.operation.authored.operation = AuthoredPrismId(897897);
    let sources = binding_sources(&fixture.operation.shapes);
    assert!(matches!(
        inspect_complete_prism_history(&fixture.request(&sources), |_| false),
        PrismHistoryOutcome::Incomplete(_)
    ));
    let fixture = HistoryFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    for maximum_rows in [0, 1, usize::MAX] {
        let mut request = fixture.request(&sources);
        request.requested.limits.maximum_rows = maximum_rows;
        assert!(matches!(
            inspect_complete_prism_history(&request, |_| false),
            PrismHistoryOutcome::Incomplete(_)
        ));
    }
}
#[test]
fn complete_prism_history_replay_across_processes_and_threads() {
    let baseline = fixture_bytes();
    let workers: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(|| [fixture_bytes(), fixture_bytes()]))
        .collect();
    for worker in workers {
        for bytes in worker.join().unwrap() {
            assert_eq!(bytes, baseline);
        }
    }
    assert_eq!(fixture_bytes(), baseline);
    let hex: String = baseline.iter().map(|b| format!("{b:02x}")).collect();
    println!("complete-prism-history-v1:{hex}");
}
#[test]
fn native_failure_keeps_typed_cause_and_previous_owned_result() {
    let mut fixture = HistoryFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    let baseline = inspected(inspect_complete_prism_history(
        &fixture.request(&sources),
        |_| false,
    ));
    let bytes = baseline.replay_bytes().to_vec();
    let crate::sketch_region::PartitionObservation::Captured(partition) =
        &mut fixture.operation.bridge.mapped.arrangement.partition
    else {
        panic!("partition");
    };
    let crate::sketch_region::PartitionMemberQueries::Captured { rows, .. } =
        &mut partition.member_queries
    else {
        panic!("queries");
    };
    rows[0].deleted = Err(std::sync::Arc::new(
        rustycad_tkernel::standard::Failure::out_of_range(
            "history query 0; expected completed deletion answer",
        ),
    ));
    let result = inspect_complete_prism_history(&fixture.request(&sources), |_| false);
    let PrismHistoryOutcome::Failed(failure) = result else {
        panic!("expected typed failed history");
    };
    assert!(failure.finding.entity.is_some());
    assert!(
        matches!(failure.finding.cause, PrismHistoryCause::AtUse { cause, .. } if matches!(*cause, PrismHistoryCause::Native(_)))
    );
    assert_eq!(baseline.replay_bytes(), bytes);
}

#[test]
fn counted_history_rows_honor_exact_boundary_and_one_short_without_clamping() {
    let fixture = HistoryFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    let mut request = fixture.request(&sources);
    let measured = budget::admit(&request).unwrap();
    request.requested.limits.maximum_rows = measured;
    let evidence = inspected(inspect_complete_prism_history(&request, |_| false));
    assert_eq!(evidence.measured_operand_rows(), measured);
    assert_eq!(evidence.requested().limits.maximum_rows, measured);
    assert!(reproject_complete_prism_history(&request, &evidence).is_ok());
    request.requested.limits.maximum_rows = measured - 1;
    assert!(matches!(
        inspect_complete_prism_history(&request, |_| false),
        PrismHistoryOutcome::Incomplete(_)
    ));
    assert!(reproject_complete_prism_history(&request, &evidence).is_err());
}
