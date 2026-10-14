// SPDX-License-Identifier: GPL-2.0-only
//! A nonzero ideal-row constant excludes a whole exceptional fiber, not just sampled points.
use super::super::{RootIsolationOptions, signs::RootSignEvaluation};
use super::{algebraic, operation::PairOperationFailure, projection::LinearFiberProjection};
use std::cmp::Ordering;
pub(super) fn certify(
    projection: &LinearFiberProjection,
    interval: [f64; 2],
    options: RootIsolationOptions,
    sign_limit: u16,
    budget: &super::work_budget::PairWorkBudget,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootSignEvaluation, PairOperationFailure> {
    let roots = algebraic::solve(&projection.alpha, interval, options, budget, cancelled)
        .map_err(PairOperationFailure::Root)?;
    let evaluation = evaluate_fibers(projection, roots, sign_limit, cancelled)?;
    if evaluation.signs().contains(&Ordering::Equal) {
        return Err(PairOperationFailure::ExceptionalFiber(evaluation));
    }
    Ok(evaluation)
}

fn evaluate_fibers(
    projection: &LinearFiberProjection,
    roots: super::super::RootIsolation,
    sign_limit: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootSignEvaluation, PairOperationFailure> {
    algebraic::query_evaluation(
        &projection.alpha,
        &projection.beta,
        &roots,
        sign_limit,
        cancelled,
    )
    .map_err(PairOperationFailure::Sign)
}
