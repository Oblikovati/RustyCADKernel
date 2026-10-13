// SPDX-License-Identifier: GPL-2.0-only
//! A nonzero ideal-row constant excludes a whole exceptional fiber, not just sampled points.
use super::super::{
    RootIsolationOptions, polynomial::IntegerPolynomial, signs, signs::RootSignEvaluation,
};
use super::{algebraic, operation::PairOperationFailure, projection::LinearFiberProjection};
use std::cmp::Ordering;
pub(super) fn certify(
    projection: &LinearFiberProjection,
    interval: [f64; 2],
    options: RootIsolationOptions,
    budget: &super::work_budget::PairWorkBudget,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootSignEvaluation, PairOperationFailure> {
    let roots = algebraic::solve(&projection.alpha, interval, options, budget, cancelled)
        .map_err(PairOperationFailure::Root)?;
    let evaluation = evaluate_fibers(projection, roots, options, cancelled)?;
    if evaluation.signs().contains(&Ordering::Equal) {
        return Err(PairOperationFailure::ExceptionalFiber(evaluation));
    }
    Ok(evaluation)
}

fn evaluate_fibers(
    projection: &LinearFiberProjection,
    roots: super::super::RootIsolation,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootSignEvaluation, PairOperationFailure> {
    let source = projection
        .alpha
        .primitive(options.max_degree)
        .map_err(PairOperationFailure::Root)?;
    let query = IntegerPolynomial::accept_zero(projection.beta.primitive(options.max_degree))
        .map_err(PairOperationFailure::Root)?;
    signs::evaluate_isolated(source, query, roots, cancelled).map_err(PairOperationFailure::Sign)
}
