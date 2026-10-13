// SPDX-License-Identifier: GPL-2.0-only
//! Retain the denominator proof tied to the actual captured homogeneous coefficients.
use super::super::{RootIsolation, RootIsolationOptions};
use super::{
    algebraic,
    operation::PairOperationFailure,
    source::{HomogeneousPolynomials, OwnedBezierSource},
};
#[derive(Debug)]
pub(super) struct SourceCertificate {
    pub homogeneous: HomogeneousPolynomials,
    pub denominator: RootIsolation,
}
pub(super) fn certify(
    source: &OwnedBezierSource,
    options: RootIsolationOptions,
    budget: &super::work_budget::PairWorkBudget,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<SourceCertificate, PairOperationFailure> {
    let homogeneous = source
        .homogeneous(cancelled)
        .map_err(PairOperationFailure::Source)?;
    let denominator =
        checked_denominator(&homogeneous, source.interval, options, budget, cancelled)?;
    Ok(SourceCertificate {
        homogeneous,
        denominator,
    })
}

fn checked_denominator(
    homogeneous: &HomogeneousPolynomials,
    interval: [f64; 2],
    options: RootIsolationOptions,
    budget: &super::work_budget::PairWorkBudget,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootIsolation, PairOperationFailure> {
    let denominator = algebraic::solve(
        &homogeneous.components[2],
        interval,
        options,
        budget,
        cancelled,
    )
    .map_err(PairOperationFailure::Root)?;
    if !denominator.roots().is_empty() {
        return Err(PairOperationFailure::DenominatorRoots(denominator));
    }
    Ok(denominator)
}
