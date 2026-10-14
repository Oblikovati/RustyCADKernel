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
    assert!(
        proof.contacts[0]
            .root
            .width()
            .compare_f64(1e-8)
            .unwrap()
            .is_lt()
    );
    assert!(outcome.work.completed > proof.candidate_certificate.isolation.bisections());
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
        Err(PairOperationFailure::WorldResolution)
    ));
}
