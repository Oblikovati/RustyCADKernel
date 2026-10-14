// SPDX-License-Identifier: GPL-2.0-only
//! Root signs certify the graph against both original equations, not approximate matching.
use super::super::{
    RootIsolation, RootIsolationError, RootIsolationOptions, checked_policy_interval,
    polynomial::IntegerPolynomial, polynomial_solve, signs,
};
use super::polynomial::ExactCoefficientPolynomial;

pub(super) fn solve(
    polynomial: &ExactCoefficientPolynomial,
    interval: [f64; 2],
    options: RootIsolationOptions,
    budget: &super::work_budget::PairWorkBudget,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    let (bounds, width) = checked_policy_interval(interval, options)?;
    let source = polynomial.primitive(options.max_degree)?;
    polynomial_solve::solve_polynomial_accounted(
        source,
        bounds,
        width,
        budget.effective(options),
        cancelled,
        &mut || budget.completed_split(),
    )
}
pub(super) fn query_evaluation(
    source: &ExactCoefficientPolynomial,
    query: &ExactCoefficientPolynomial,
    isolation: &RootIsolation,
    degree_limit: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<signs::RootSignEvaluation, signs::RootSignError> {
    let source = source.primitive(degree_limit).map_err(source_failure)?;
    let query =
        IntegerPolynomial::accept_zero(query.primitive(degree_limit)).map_err(query_failure)?;
    signs::evaluate_isolated(source, query, isolation.clone(), cancelled)
}
fn source_failure(cause: RootIsolationError) -> signs::RootSignError {
    signs::RootSignError::Root {
        stage: signs::RootSignStage::Source,
        cause,
    }
}
fn query_failure(cause: RootIsolationError) -> signs::RootSignError {
    signs::RootSignError::Root {
        stage: signs::RootSignStage::Query,
        cause,
    }
}
pub(super) fn graph_boundary(
    alpha: &ExactCoefficientPolynomial,
    beta: &ExactCoefficientPolynomial,
    bound: f64,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactCoefficientPolynomial, RootIsolationError> {
    let unit = crate::predicates::exact_coordinate(1., 0, 0).expect("finite unit");
    let bound = crate::predicates::exact_coordinate(bound, 0, 0).map_err(|_| {
        RootIsolationError::Coefficient {
            index: 0,
            value: bound,
        }
    })?;
    let numerator = beta.scaled(&(-unit)).subtract(&alpha.scaled(&bound));
    numerator.multiply(alpha, cancelled)
}
pub(super) fn off_diagonal(
    alpha: &ExactCoefficientPolynomial,
    beta: &ExactCoefficientPolynomial,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactCoefficientPolynomial, RootIsolationError> {
    let numerator = beta
        .scaled(&(-num_bigint::BigInt::from(1)))
        .subtract(&alpha.shifted());
    numerator.multiply(alpha, cancelled)
}
