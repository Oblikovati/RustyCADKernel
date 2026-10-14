// SPDX-License-Identifier: GPL-2.0-only
use super::tests::{candidates, cubic, rectangle};
use super::{captured_store::CandidateCaptureReceipt, *};
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
fn classified(batch: &CandidateBatch, seed: [f64; 2]) -> CandidateClassification {
    classify_candidates(batch, seed, RootIsolationOptions::new(0.001), || false).unwrap_or_else(
        |_| panic!("expected incomplete candidate selection with actual captured proofs"),
    )
}
fn receipt(result: &CandidateClassification) -> &CandidateCaptureReceipt {
    let CandidateClassification::Incomplete { captured, .. } = result;
    captured
}
#[test]
fn actual_opposite_cubic_candidates_reuse_one_physical_certificate() {
    let batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let result = classified(&batch, [1., 1.]);
    let captured = receipt(&result);
    assert_eq!(captured.store.proofs().len(), 1);
    assert_eq!(captured.occurrences.len(), 2);
    assert_eq!(captured.work.reused_certificates, 1);
    assert_eq!(
        captured.work.new_bisections,
        captured.work.operation_bisections
    );
    assert_opposite_occurrences(captured);
}
fn assert_opposite_occurrences(captured: &CandidateCaptureReceipt) {
    assert_eq!(
        captured.occurrences[0].physical_source,
        captured.occurrences[1].physical_source
    );
    assert_ne!(
        captured.occurrences[0].reversed,
        captured.occurrences[1].reversed
    );
    assert!(
        captured
            .occurrences
            .iter()
            .all(|entry| !entry.parents.is_empty())
    );
}

#[test]
fn boundary_attribution_retains_both_current_occurrence_ids_on_reuse() {
    let batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let result = classified(&batch, [0., 0.]);
    let CandidateClassification::Incomplete {
        selection: SeedSelection::Boundary { occurrences },
        captured,
        ..
    } = result
    else {
        panic!("expected exact original endpoint boundary");
    };
    let expected: Vec<_> = captured.occurrences.iter().map(|entry| entry.id).collect();
    assert_eq!(occurrences, expected);
    assert_eq!(captured.work.reused_certificates, 1);
}
#[test]
fn missing_certificates_remove_only_the_proved_reuse_obligation() {
    let batch = candidates(&rectangle(0., 0., 4., 4.));
    let CandidateClassification::Incomplete {
        missing,
        achieved_error,
        ..
    } = classified(&batch, [1., 1.]);
    assert_eq!(
        missing,
        vec![
            MissingCertificate::Construction,
            MissingCertificate::SearchCompleteness,
            MissingCertificate::SelfIntersection,
            MissingCertificate::Tolerance
        ]
    );
    assert_eq!(achieved_error, None);
}
#[test]
fn one_operation_charges_cold_work_once_but_retains_historical_proof_cost() {
    let batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let result = classified(&batch, [1., 1.]);
    let captured = receipt(&result);
    let proof = &captured.store.proofs()[0];
    assert_eq!(captured.work.new_bisections, proof.bisections());
    assert_eq!(
        captured.work.certificate_bisections,
        2 * u64::from(proof.bisections())
    );
    assert!(proof.denominator().roots().is_empty());
}
#[test]
fn independent_operation_does_not_reuse_previous_seed_or_policy_proofs() {
    let batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let first = classified(&batch, [1., 1.]);
    let second = classified(&batch, [2., 1.]);
    assert_eq!(receipt(&first).store.proofs().len(), 1);
    assert_eq!(receipt(&second).store.proofs().len(), 1);
    assert!(receipt(&first).work.new_bisections > 0);
    assert!(receipt(&second).work.new_bisections > 0);
}
#[test]
fn consumed_cold_budget_still_allows_opposite_occurrence_replay() {
    let batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let measured = classified(&batch, [1., 1.]);
    let budget = receipt(&measured).work.new_bisections;
    let options = budget_policy(budget);
    let result = classify_candidates(&batch, [1., 1.], options, || false)
        .unwrap_or_else(|_| panic!("expected no new numerical work for the opposite occurrence"));
    let CandidateClassification::Incomplete {
        remaining,
        captured,
        selection,
        ..
    } = result;
    assert_eq!(remaining.max_bisections, 0);
    assert_eq!(captured.work.new_bisections, budget);
    assert_eq!(captured.work.reused_certificates, 1);
    assert!(matches!(selection, SeedSelection::Unique { .. }));
}
#[test]
fn failed_attempt_keeps_actual_work_and_complete_proof_prefix() {
    let batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let options = budget_policy(1);
    let Err(CandidateCause::CapturedOperation(retained)) =
        classify_candidates(&batch, [1., 1.], options, || false)
    else {
        panic!("expected actual one-split work exhaustion");
    };
    assert_eq!(retained.retained.work.new_bisections, 1);
    assert_eq!(retained.retained.work.operation_bisections, 1);
    assert!(retained.retained.store.proofs().is_empty());
    assert!(matches!(
        retained.cause.as_ref(),
        CandidateCause::CapturedNumerical(_)
    ));
}
#[test]
fn retained_owned_proof_survives_later_candidate_flags_and_scalar_mutation() {
    let mut batch = candidates(&[cubic::closed_analytic_cubic(1., false)]);
    let first = classified(&batch, [1., 1.]);
    let old_cost = receipt(&first).work;
    let changed = &mut batch.candidates[0].wires[0].segments[0];
    changed.controls[1][0] = 8.;
    changed.edge.set_closed(false).unwrap();
    assert_eq!(receipt(&first).work, old_cost);
    assert_eq!(receipt(&first).store.proofs().len(), 1);
    assert!(
        receipt(&first).store.proofs()[0]
            .denominator()
            .roots()
            .is_empty()
    );
    let second = classified(&batch, [1., 1.]);
    assert_eq!(receipt(&second).store.proofs().len(), 2);
    assert_eq!(receipt(&second).work.reused_certificates, 0);
}

fn budget_policy(max_bisections: u16) -> RootIsolationOptions {
    RootIsolationOptions {
        max_bisections,
        ..RootIsolationOptions::new(0.001)
    }
}
