// SPDX-License-Identifier: GPL-2.0-only
//! Captured source lifetime, mode, policy and caller cancellation remain separate from results.
use super::super::RootIsolationOptions;
use super::{query::*, sign_policy::PairSignOptions, source::OwnedBezierSource};
fn policy(version: u32) -> PairQueryPolicy {
    PairQueryPolicy {
        version,
        roots: RootIsolationOptions::new(1e-9),
        signs: PairSignOptions::derived(),
        requested_world_error: 1e-6,
    }
}
fn distinct() -> CapturedBezierPair {
    let source =
        |controls: &[[f64; 2]]| OwnedBezierSource::capture(controls, None, [0., 1.]).unwrap();
    CapturedBezierPair::distinct(
        source(&[[0., 0.], [0.5, 0.], [1., 1.]]),
        source(&[[0., 0.5], [0.5, 0.5], [1., -0.5]]),
    )
}
#[test]
fn actual_captured_query_binds_result_after_success_failure_and_source_mutation() {
    let mut controls = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let query = CapturedBezierPair::self_off_diagonal(&controls, None, [0., 1.]).unwrap();
    controls.fill([f64::NAN, f64::NAN]);
    let success = query.evaluate(policy(1), &mut || false);
    assert_eq!(success.query, query);
    assert_eq!(success.policy, policy(1));
    assert_eq!(outcome_contact_count(success), 1);
    assert_cancelled(&query);
    assert_eq!(query.sources()[0].controls[1], [4., 0.]);
    assert_eq!(
        outcome_contact_count(query.evaluate(policy(1), &mut || false)),
        1
    );
}
#[test]
fn unknown_outer_policy_version_does_not_poll_or_return_successful_empty() {
    let query = distinct();
    let mut polled = false;
    let failure = query.evaluate(policy(77), &mut || {
        polled = true;
        false
    });
    assert_unknown_version(&failure);
    assert!(!polled);
    assert_eq!(failure.query, query);
    assert!(
        query
            .evaluate(policy(1), &mut || false)
            .outcome
            .unwrap()
            .completion
            .is_ok()
    );
}
#[test]
fn foreign_callback_failure_is_not_masked_and_captured_query_can_retry() {
    let query = distinct();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        query.evaluate(policy(1), &mut || std::panic::panic_any(77u32))
    }));
    assert_eq!(*failure.err().unwrap().downcast::<u32>().unwrap(), 77);
    assert!(
        query
            .evaluate(policy(1), &mut || false)
            .outcome
            .unwrap()
            .completion
            .is_ok()
    );
}

fn outcome_contact_count(outcome: CapturedPairOutcome) -> usize {
    let accounted = outcome.outcome.unwrap();
    let completed = accounted.completion.unwrap();
    completed.contacts.len()
}
#[test]
fn complete_extrapolated_pair_contacts_keep_distinct_candidate_root_identity() {
    let source =
        |controls: &[[f64; 2]]| OwnedBezierSource::capture(controls, None, [-1., 1.]).unwrap();
    let query = CapturedBezierPair::distinct(
        source(&[[0., 0.], [0.5, 0.], [1., 1.]]),
        source(&[[0., 0.5], [0.5, 0.5], [1., -0.5]]),
    );
    let proof = completed_query(&query);
    assert_eq!(proof.contacts.len(), 2);
    for (index, parameter) in [(0, -0.5), (1, 0.5)] {
        let contact = &proof.contacts[index];
        assert_eq!(contact.candidate_index, index);
        assert_eq!(
            contact.root.bounds().0.compare_f64(parameter).unwrap(),
            std::cmp::Ordering::Equal
        );
        assert_eq!(contact.u_bounds, [parameter, parameter]);
        assert_eq!(contact.world.point, [parameter, 0.25]);
    }
}
#[test]
fn sign_separated_negative_rational_weights_preserve_actual_contact_and_source() {
    let left =
        OwnedBezierSource::capture(&[[0., 0.], [0.5, 1.]], Some(&[-1., -2.]), [0., 1.]).unwrap();
    let right = OwnedBezierSource::capture(&[[0., 1.], [1., 0.]], None, [0., 1.]).unwrap();
    let query = CapturedBezierPair::distinct(left, right);
    let proof = completed_query(&query);
    assert_eq!(proof.contacts.len(), 1);
    assert_eq!(
        proof.contacts[0].root.bounds().0.compare_f64(0.5).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(
        proof.sources[0].weights.as_deref(),
        Some([-1., -2.].as_slice())
    );
    assert!(proof.source_certificates[0].denominator.roots().is_empty());
}

fn completed_query(query: &CapturedBezierPair) -> super::operation::PairOperationProof {
    let outcome = query.evaluate(policy(1), &mut || false);
    let accounted = outcome.outcome.unwrap();
    accounted.completion.unwrap()
}
fn assert_cancelled(query: &CapturedBezierPair) {
    let outcome = query.evaluate(policy(1), &mut || true);
    let completion = outcome.outcome.unwrap().completion;
    assert!(completion.is_err());
}

fn assert_unknown_version(failure: &CapturedPairOutcome) {
    let result = &failure.outcome;
    let expected = PairQueryPreparationFailure::Version(77);
    assert_eq!(result.as_ref().err(), Some(&expected));
}
