// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use RootIsolationError as IsolationFailure;
use RootSignStage::{Policy, Query, Source};
use std::error::Error;
#[test]
fn algebraic_zero_source_and_invalid_version_retain_their_input_owner() {
    let zero = IsolationFailure::ZeroPolynomial { coefficients: 0 };
    assert_failure(&[], &[1.], options(0.1), Source, zero);
    let mut policy = options(0.1);
    policy.version = 2;
    assert_failure(
        &[1.],
        &[0.],
        policy,
        Policy,
        IsolationFailure::OptionsVersion(2),
    );
}
#[test]
fn algebraic_nonfinite_coefficients_name_the_source_or_query() {
    let query = IsolationFailure::Coefficient {
        index: 0,
        value: f64::INFINITY,
    };
    assert_failure(&[0., 1.], &[f64::INFINITY], options(0.1), Query, query);
    let source = IsolationFailure::Coefficient {
        index: 0,
        value: f64::NEG_INFINITY,
    };
    assert_failure(&[f64::NEG_INFINITY], &[0.], options(0.1), Source, source);
}
fn assert_failure(
    source: &[f64],
    query: &[f64],
    policy: RootIsolationOptions,
    stage: RootSignStage,
    cause: IsolationFailure,
) {
    let error = signs_at_polynomial_roots(source, query, [-2., 2.], policy, || false).unwrap_err();
    assert!(error.to_string().contains("expected"));
    assert!(error.source().is_some());
    assert_eq!(error, RootSignError::Root { stage, cause });
}
#[test]
fn algebraic_query_and_source_degree_are_admitted_independently() {
    let mut policy = options(0.001);
    policy.max_degree = 1;
    let degree = IsolationFailure::Degree {
        actual: 2,
        limit: 1,
    };
    assert_failure(&[0., 1.], &[0., 0., 1.], policy, Query, degree.clone());
    assert_failure(&[-2., 0., 1.], &[0., 1.], policy, Source, degree);
}
#[test]
fn algebraic_sign_classification_does_not_widen_the_subdivision_budget() {
    let mut policy = options(0.001);
    policy.max_bisections = 0;
    let exhausted = IsolationFailure::SubdivisionLimit { limit: 0 };
    assert_failure(&[-2., 0., 1.], &[0., 1.], policy, Source, exhausted);
}
#[test]
fn algebraic_sign_cancellation_discards_every_partial_sign_list() {
    for query in [
        vec![0., 1.],
        vec![-2., 0., 1.],
        vec![0.],
        vec![1., -2., 3., -4., 5.],
    ] {
        check_cancellation(&query);
    }
}
fn check_cancellation(query: &[f64]) {
    let source = [-4., 4., 4., -4., -1., 1.];
    let mut checkpoints = 0;
    let expected = signs_at_polynomial_roots(&source, query, [-2., 2.], options(0.01), || {
        checkpoints += 1;
        false
    })
    .unwrap();
    for stop in 1..=checkpoints {
        cancel_at(&source, query, stop);
    }
    assert_eq!(classify(&source, query, [-2., 2.], 0.01), expected);
}
fn cancel_at(source: &[f64], query: &[f64], stop: usize) {
    let mut calls = 0;
    let result = signs_at_polynomial_roots(source, query, [-2., 2.], options(0.01), || {
        calls += 1;
        calls == stop
    });
    assert!(
        matches!(
            result,
            Err(RootSignError::Root {
                cause: IsolationFailure::Cancelled,
                ..
            })
        ),
        "checkpoint {stop}: {result:?}"
    );
}
#[test]
fn algebraic_sign_sources_and_completed_results_are_owned_across_failures_and_edits() {
    let mut source = vec![-2., -0., 1.];
    let mut query = vec![0., 1.];
    let before = source_bits(&source, &query);
    let completed = classify(&source, &query, [-2., 2.], 0.001);
    let snapshot = completed.clone();
    readonly_failures(&source, &query);
    assert_eq!(source_bits(&source, &query), before);
    source.fill(1.);
    query.fill(-1.);
    source.clear();
    query.clear();
    assert_eq!(completed, snapshot);
}
fn readonly_failures(source: &[f64], query: &[f64]) {
    assert!(signs_at_polynomial_roots(source, query, [-2., 2.], options(0.001), || true).is_err());
    let mut exhausted = options(0.001);
    exhausted.max_bisections = 0;
    assert!(signs_at_polynomial_roots(source, query, [-2., 2.], exhausted, || false).is_err());
}
fn source_bits(source: &[f64], query: &[f64]) -> Vec<u64> {
    source
        .iter()
        .chain(query)
        .map(|value| value.to_bits())
        .collect()
}
#[test]
fn algebraic_variation_failure_explains_the_required_one_root_invariant() {
    let error = RootSignError::Variation { lower: 0, upper: 2 };
    assert!(
        error
            .to_string()
            .contains("expected difference -1, 0, or 1")
    );
    assert!(error.source().is_none());
}
