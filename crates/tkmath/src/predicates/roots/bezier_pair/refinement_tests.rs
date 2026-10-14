// SPDX-License-Identifier: GPL-2.0-only
//! Construction limits require source-bound refinement, never smaller recorded errors.
use super::super::RootIsolationOptions;
use super::{operation::*, source::OwnedBezierSource};
fn source(controls: &[[f64; 2]], weights: Option<&[f64]>) -> OwnedBezierSource {
    OwnedBezierSource::capture(controls, weights, [0., 1.]).unwrap()
}
fn irrational_sources() -> [OwnedBezierSource; 2] {
    [
        source(&[[0., 0.], [0.5, 0.], [1., 1.]], None),
        source(&[[0., 1.], [0.5, 1.], [1., 0.]], None),
    ]
}
#[test]
fn world_request_refines_the_same_joint_root_beyond_initial_parameter_width() {
    let outcome = compute_accounted(
        irrational_sources(),
        false,
        RootIsolationOptions::new(0.01),
        1e-8,
        &mut || false,
    );
    let proof = outcome.completion.unwrap();
    assert_eq!(proof.contacts.len(), 1);
    assert!(proof.contacts[0].world.achieved_error <= 1e-8);
    assert!(root_width_less_than(&proof.contacts[0].root, 1e-8));
    let initial = &proof.graph().unwrap().candidate_certificate.isolation;
    assert!(outcome.work.completed > initial.bisections());
    assert_root_link(&proof);
    super::algebraic_world_tests::verify_exact_sqrt_half_bracket(&proof.contacts[0].root);
    super::algebraic_world_tests::verify_independent_world_error(&proof.contacts[0], 1e-8);
}
#[test]
fn refinement_exhaustion_is_incomplete_with_actual_spent_global_work() {
    let mut options = RootIsolationOptions::new(0.01);
    options.max_bisections = 8;
    let outcome = compute_accounted(irrational_sources(), false, options, 1e-8, &mut || false);
    assert!(matches!(
        outcome.completion,
        Err(PairOperationFailure::Root(
            super::super::RootIsolationError::SubdivisionLimit { .. }
        ))
    ));
    assert_eq!(outcome.work.completed, 8);
}
fn rational_third_sources() -> [OwnedBezierSource; 2] {
    [
        source(&[[0., 0.], [0.5, 1.]], Some(&[1., 2.])),
        source(&[[0., 1.], [1., 0.]], None),
    ]
}
#[test]
fn singleton_nonbinary_world_point_meets_certifiable_request_but_not_tighter_one() {
    let options = RootIsolationOptions::new(1e-9);
    let accepted = compute(rational_third_sources(), false, options, 5e-17, &mut || {
        false
    })
    .unwrap();
    assert_eq!(
        accepted.contacts[0].root.width().compare_f64(0.).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert!(accepted.contacts[0].world.achieved_error > 0.);
    assert!(accepted.contacts[0].world.achieved_error <= 5e-17);
    let incomplete = compute(rational_third_sources(), false, options, 1e-18, &mut || {
        false
    });
    assert!(matches!(
        incomplete,
        Err(PairOperationFailure::WorldResolution { .. })
    ));
}

fn root_width_less_than(root: &super::super::RootBracket, limit: f64) -> bool {
    let width = root.width();
    let order = width.compare_f64(limit).unwrap();
    order.is_lt()
}

fn assert_root_link(proof: &PairOperationProof) {
    let contact = &proof.contacts[0];
    let original = &proof
        .graph()
        .unwrap()
        .candidate_certificate
        .isolation
        .roots()[contact.candidate_index];
    let refined = &proof.contacts[0].root;
    assert!(refined.bounds().0 >= original.bounds().0);
    assert!(refined.bounds().1 <= original.bounds().1);
    assert_eq!(refined.multiplicity(), original.multiplicity());
}
#[test]
fn singleton_limit_retains_requested_error_and_constructed_root_evidence() {
    let cause = singleton_limit();
    let PairOperationFailure::WorldResolution {
        requested_error,
        last_root,
        enclosure,
    } = cause
    else {
        panic!("expected singleton construction limit, got {cause:?}");
    };
    assert_eq!(requested_error, 1e-18);
    assert_eq!(last_root.bounds().0, last_root.bounds().1);
    assert_resolution_enclosure(enclosure.unwrap(), requested_error);
}
fn counted_irrational(cancel_after: usize) -> (AccountedPairOutcome, usize) {
    let mut polls = 0;
    let outcome = compute_accounted(
        irrational_sources(),
        false,
        RootIsolationOptions::new(0.01),
        1e-8,
        &mut || {
            polls += 1;
            polls > cancel_after
        },
    );
    (outcome, polls)
}
#[test]
fn refinement_cancellation_retains_work_and_fresh_retry_is_identical() {
    let (success, polls) = counted_irrational(usize::MAX);
    let (cancelled, _) = counted_irrational(polls - 5);
    assert!(cancelled.completion.is_err());
    assert!(
        cancelled.work.completed
            > success
                .completion
                .as_ref()
                .unwrap()
                .graph()
                .unwrap()
                .candidate_certificate
                .isolation
                .bisections()
    );
    let (retry, _) = counted_irrational(usize::MAX);
    assert_eq!(retry.work, success.work);
    assert_retry_root(retry, success);
}

fn assert_resolution_enclosure(enclosure: ([[f64; 2]; 2], [f64; 2], f64), requested: f64) {
    let (bounds, point, achieved) = enclosure;
    assert!(achieved > requested);
    assert!(bounds[0][0] <= point[0] && point[0] <= bounds[0][1]);
}

fn assert_retry_root(retry: AccountedPairOutcome, original: AccountedPairOutcome) {
    let retry = retry.completion.unwrap();
    let original = original.completion.unwrap();
    assert_eq!(retry.contacts[0].root, original.contacts[0].root);
}

fn singleton_limit() -> PairOperationFailure {
    compute(
        rational_third_sources(),
        false,
        RootIsolationOptions::new(1e-9),
        1e-18,
        &mut || false,
    )
    .err()
    .unwrap()
}
