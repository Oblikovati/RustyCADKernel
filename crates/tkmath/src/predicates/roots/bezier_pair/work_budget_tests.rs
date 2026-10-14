// SPDX-License-Identifier: GPL-2.0-only
//! Counts are actual completed subdivisions across stages, including terminal failure work.
use super::super::{RootIsolationError, RootIsolationOptions};
use super::{
    algebraic, operation::*, polynomial::ExactCoefficientPolynomial, source::OwnedBezierSource,
    work_budget::PairWorkBudget,
};
fn sources() -> [OwnedBezierSource; 2] {
    let a = OwnedBezierSource::capture(&[[0., 0.], [0., 0.5], [1., 1.]], None, [0., 1.]).unwrap();
    let b =
        OwnedBezierSource::capture(&[[0., 0.], [0.25, 0.5]], Some(&[1., 2.]), [0., 1.]).unwrap();
    [a, b]
}
fn run(limit: u16) -> AccountedPairOutcome {
    let mut options = RootIsolationOptions::new(1e-9);
    options.max_bisections = limit;
    compute_accounted(sources(), false, options, 1e-6, &mut || false)
}
#[test]
fn one_global_budget_does_not_reset_between_exceptional_and_candidate_isolation() {
    // sqrt(1/2) needs30 halvings to width2^-30<=1e-9; candidate t=1/2 needs1more.
    let exhausted = run(30);
    assert_eq!(exhausted.work.requested, 30);
    assert_eq!(exhausted.work.completed, 30);
    assert!(matches!(
        exhausted.completion,
        Err(PairOperationFailure::Root(
            RootIsolationError::SubdivisionLimit { limit: 0 }
        ))
    ));
    let complete = run(31);
    assert_eq!(complete.work.completed, 31);
    let proof = complete.completion.unwrap();
    assert_eq!(proof.work, complete.work);
    let graph = proof.graph().unwrap();
    assert_eq!(graph.exceptional_fibers.isolation().bisections(), 30);
    assert_eq!(graph.candidate_certificate.isolation.bisections(), 1);
    assert_eq!(proof.contacts.len(), 2);
}
#[test]
fn cancelled_and_exhausted_stages_still_charge_the_successful_split() {
    let polynomial = ExactCoefficientPolynomial::new(vec![(-2).into(), 0.into(), 1.into()]);
    let budget = PairWorkBudget::new(1);
    let options = RootIsolationOptions::new(1e-9);
    let error =
        algebraic::solve(&polynomial, [0., 2.], options, &budget, &mut || false).unwrap_err();
    assert_eq!(error, RootIsolationError::SubdivisionLimit { limit: 1 });
    assert_eq!(budget.receipt().completed, 1);
    let cancelled = PairWorkBudget::new(8);
    let error = algebraic::solve(&polynomial, [0., 2.], options, &cancelled, &mut || {
        cancelled.receipt().completed == 1
    })
    .unwrap_err();
    assert_eq!(error, RootIsolationError::Cancelled);
    assert_eq!(cancelled.receipt().completed, 1);
}
#[test]
fn zero_budget_still_completes_exact_endpoint_work_without_splitting() {
    let polynomial = ExactCoefficientPolynomial::new(vec![0.into(), 1.into()]);
    let budget = PairWorkBudget::new(0);
    let roots = algebraic::solve(
        &polynomial,
        [0., 1.],
        RootIsolationOptions::new(1e-9),
        &budget,
        &mut || false,
    )
    .unwrap();
    assert_eq!(roots.roots().len(), 1);
    assert_eq!(roots.bisections(), 0);
    assert_eq!(budget.receipt().completed, 0);
}
