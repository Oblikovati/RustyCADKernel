// SPDX-License-Identifier: GPL-2.0-only
//! Native empty/false/error answers, budgets, source mutation and isolation stay distinct outcomes.
use super::adversarial_tests::{assert_refused, refused};
use super::fixture::AnnularHistoryCase;
use super::operation_tests::inspected;
use super::*;
use crate::sketch_region::{PartitionMemberQueries, PartitionMemberQuery, PartitionObservation};

fn queries(case: &mut AnnularHistoryCase) -> &mut Vec<PartitionMemberQuery> {
    let partition = &mut case.fixture.bridge.mapped.arrangement.partition;
    let PartitionObservation::Captured(partition) = partition else {
        panic!("expected captured partition");
    };
    let PartitionMemberQueries::Captured { rows, .. } = &mut partition.member_queries else {
        panic!("expected captured member queries");
    };
    rows
}
#[test]
fn same_cardinality_query_omission_refuses_exhaustive_query_binding() {
    let mut case = AnnularHistoryCase::nested();
    let original = queries(&mut case)[1].clone();
    queries(&mut case)[1] = queries(&mut case)[0].clone();
    assert_refused(
        &case,
        "one actual native partition query for each original use",
    );
    queries(&mut case)[1] = original;
    inspected(&case);
}
#[test]
fn true_deletion_and_native_error_answers_refuse_with_distinct_classes() {
    let mut case = AnnularHistoryCase::nested();
    let original = queries(&mut case)[1].clone();
    queries(&mut case)[1].deleted = Ok(true);
    assert_refused(&case, "v1 continued original member is not deleted");
    queries(&mut case)[1].deleted = original.deleted.clone();
    let failure = rustycad_tkernel::standard::Failure::new("fixture native query failure");
    queries(&mut case)[1].modified = Err(std::sync::Arc::new(failure));
    assert_eq!(refused(&case).class, PrismHistoryFailureClass::Failed);
    queries(&mut case)[1] = original;
    inspected(&case);
}
#[test]
fn changed_sweep_retirement_answer_refuses_exhaustive_query_correspondence() {
    let mut case = AnnularHistoryCase::nested();
    let diagnostic = case.receipt.diagnostic.as_mut().unwrap();
    let crate::sketch_region_prism::PrismSweepQueries::Captured(rows) =
        &mut diagnostic.sweep_queries
    else {
        panic!("expected exhaustive sweep queries");
    };
    rows.iter_mut().find(|r| !r.retired).unwrap().retired = true;
    assert_eq!(refused(&case).class, PrismHistoryFailureClass::Incomplete);
}
#[test]
fn exact_one_short_and_invalid_budgets_are_typed_without_truncation() {
    let mut case = AnnularHistoryCase::nested();
    let measured = inspected(&case).measured_operand_rows();
    case.names.requested.limits.maximum_rows = measured;
    assert_eq!(inspected(&case).measured_operand_rows(), measured);
    case.names.requested.limits.maximum_rows = measured - 1;
    let refusal = refused(&case);
    assert!(refusal.cause.contains("Exhausted"), "{}", refusal.cause);
    assert_eq!(refusal.phase, PrismHistoryPhase::Admission);
    case.names.requested.limits.maximum_rows = 65537;
    assert!(refused(&case).cause.contains("Setting"));
    case.names.requested = PrismHistoryOptions::v1();
    assert_refused(&case, "complete annular history behavior version");
}
#[test]
fn source_mutation_after_inspection_refuses_final_rebind() {
    let mut case = AnnularHistoryCase::nested();
    let history = inspected(&case);
    let original = case.fixture.inner.clone();
    case.fixture.inner = original.reversed();
    let rebound =
        |c: &AnnularHistoryCase| c.run(|r| rebind_annular_prism_history(r, &history).is_ok());
    assert!(!rebound(&case));
    case.fixture.inner = original;
    assert!(rebound(&case));
}
#[test]
fn refusal_and_cancellation_leave_later_inspection_identical_to_fresh() {
    let mut case = AnnularHistoryCase::nested();
    let fresh = inspected(&AnnularHistoryCase::nested());
    let cancel = |p: PrismHistoryPhase| p == PrismHistoryPhase::Capture;
    let cancelled = case.run(|request| {
        let outcome = inspect_annular_prism_history(request, cancel);
        matches!(outcome, AnnularHistoryOutcome::Cancelled { .. })
    });
    assert!(cancelled);
    case.names.requested.version = 3;
    refused(&case);
    case.names.requested.version = 2;
    assert_eq!(inspected(&case).replay_bytes(), fresh.replay_bytes());
}
#[test]
fn independent_threads_produce_identical_history_bytes() {
    let run = || {
        inspected(&AnnularHistoryCase::nested())
            .replay_bytes()
            .to_vec()
    };
    let workers: Vec<_> = (0..2).map(|_| std::thread::spawn(run)).collect();
    let bytes: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(bytes[0], bytes[1]);
}
