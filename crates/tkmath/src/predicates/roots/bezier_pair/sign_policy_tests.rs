// SPDX-License-Identifier: GPL-2.0-only
//! Generated sign admission is separate from every primary root polynomial admission.
use super::super::RootIsolationOptions;
use super::{operation::*, sign_policy::*, source::OwnedBezierSource};
fn curved_sources() -> [OwnedBezierSource; 2] {
    let controls = [
        vec![[0., 0.], [0.5, 0.], [1., 1.]],
        vec![[0., 0.5], [0.5, 0.5], [1., -0.5]],
    ];
    controls.map(|points| OwnedBezierSource::capture(&points, None, [0., 1.]).unwrap())
}
fn run_signs(signs: PairSignOptions, primary_limit: u16) -> AccountedPairOutcome {
    let mut options = RootIsolationOptions::new(1e-9);
    options.max_degree = primary_limit;
    compute_with_signs(
        curved_sources(),
        (false, false),
        options,
        signs,
        1e-6,
        &mut || false,
    )
}
#[test]
fn derived_sign_receipt_retains_version_effective_bound_and_primary_policy() {
    let proof = run_signs(PairSignOptions::derived(), 2).completion.unwrap();
    assert_eq!(proof.graph().unwrap().signs.requested.version, 1);
    assert_eq!(
        proof.graph().unwrap().signs.requested.degree_limit,
        PairSignAdmission::DerivedProjection
    );
    assert_eq!(
        proof.graph().unwrap().signs.conservative_generated_degree,
        2
    );
    assert_eq!(proof.graph().unwrap().signs.effective_limit, 2);
    assert_eq!(proof.contacts.len(), 1);
}
#[test]
fn explicit_sign_limit_fails_without_silently_using_the_root_degree_cap() {
    let options = PairSignOptions {
        version: 1,
        degree_limit: PairSignAdmission::Explicit(1),
    };
    let outcome = run_signs(options, 2);
    assert!(matches!(
        outcome.completion,
        Err(PairOperationFailure::Sign(_))
    ));
    assert!(run_signs(PairSignOptions::derived(), 2).completion.is_ok());
}
#[test]
fn derived_sign_admission_does_not_widen_primary_candidate_isolation() {
    let outcome = run_signs(PairSignOptions::derived(), 1);
    assert!(matches!(
        outcome.completion,
        Err(PairOperationFailure::Root(
            super::super::RootIsolationError::Degree { .. }
        ))
    ));
    assert!(run_signs(PairSignOptions::derived(), 2).completion.is_ok());
}
#[test]
fn unknown_generated_sign_version_is_typed_incomplete_and_repair_reuses_source() {
    let options = PairSignOptions {
        version: 77,
        degree_limit: PairSignAdmission::DerivedProjection,
    };
    let outcome = run_signs(options, 2);
    assert!(matches!(
        outcome.completion,
        Err(PairOperationFailure::SignPolicy(
            PairSignPolicyFailure::Version(77)
        ))
    ));
    assert!(run_signs(PairSignOptions::derived(), 2).completion.is_ok());
}
fn low_root_high_domain_sources() -> [OwnedBezierSource; 2] {
    // Integer Bernstein controls exactly encode A=(12t²,144t⁴+12t), B=(12u,144u²).
    // The original joint ideal gives u=t² and 12t=0, while u endpoint signs stay quadratic.
    let controls = [
        vec![[0., 0.], [0., 3.], [2., 6.], [6., 9.], [12., 156.]],
        vec![[0., 0.], [6., 0.], [12., 144.]],
    ];
    controls.map(|points| OwnedBezierSource::capture(&points, None, [0., 1.]).unwrap())
}
fn low_root_high_domain(signs: PairSignOptions) -> AccountedPairOutcome {
    let mut options = RootIsolationOptions::new(1e-9);
    options.max_degree = 1;
    compute_with_signs(
        low_root_high_domain_sources(),
        (false, false),
        options,
        signs,
        1e-6,
        &mut || false,
    )
}
#[test]
fn genuine_quartic_pair_records_linear_primary_and_quadratic_domain_admission() {
    let proof = low_root_high_domain(PairSignOptions::derived())
        .completion
        .unwrap();
    assert_eq!(
        proof
            .graph()
            .unwrap()
            .candidate_certificate
            .polynomial
            .degree(),
        Some(1)
    );
    assert_eq!(proof.graph().unwrap().signs.effective_limit, 2);
    assert_eq!(proof.contacts.len(), 1);
    assert_eq!(proof.contacts[0].u_bounds, [0., 0.]);
    assert_eq!(proof.contacts[0].world.point, [0., 0.]);
}
#[test]
fn genuine_quartic_domain_degree_exhaustion_is_incomplete_then_repair_completes() {
    let signs = PairSignOptions {
        version: 1,
        degree_limit: PairSignAdmission::Explicit(1),
    };
    let outcome = low_root_high_domain(signs);
    assert!(matches!(
        outcome.completion,
        Err(PairOperationFailure::Sign(_))
    ));
    assert!(
        low_root_high_domain(PairSignOptions::derived())
            .completion
            .is_ok()
    );
}
