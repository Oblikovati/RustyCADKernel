// SPDX-License-Identifier: GPL-2.0-only
//! Shared isolation entry for every derived equation; the solver is the ADR 0004 engine.
use super::super::{
    DyadicBound, RootBracket, RootIsolation, polynomial::IntegerPolynomial, polynomial_solve,
};
use super::algebraic::{RootEquation, left_sign};
use super::budget::Work;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::report::SketchContactTransition;
use num_bigint::{BigInt, Sign};
use std::sync::Arc;

/// Complete isolation of a nonzero equation on closed exact bounds.
pub(super) fn isolate(
    polynomial: &IntegerPolynomial,
    bounds: [DyadicBound; 2],
    work: &mut Work<'_>,
) -> Result<(Arc<RootEquation>, RootIsolation), Cause> {
    // The whole interval is an admissible width: isolation stops at one root per
    // bracket; construction refinement is charged to the separate refinement limit.
    work.enter(SketchIncidenceStage::Isolation);
    let width = bounds[1].difference(&bounds[0]);
    let options = work.isolation();
    let isolation = polynomial_solve::solve_polynomial(
        polynomial.clone(),
        bounds,
        width,
        options,
        &mut || work.poll(),
    )?;
    Ok((RootEquation::new(polynomial.clone(), work)?, isolation))
}
/// Symmetric dyadic bounds strictly containing every real root (Cauchy's bound).
pub(super) fn cauchy_bounds(polynomial: &IntegerPolynomial) -> [DyadicBound; 2] {
    let leading = polynomial.0[polynomial.degree()].magnitude().bits() as i64;
    let others = polynomial.0[..polynomial.degree()]
        .iter()
        .map(|value| value.magnitude().bits() as i64)
        .max()
        .unwrap_or(0);
    // |c_i / c_n| < 2^(others - leading + 1), so 1 + max ratio < 2^(others - leading + 2).
    let exponent = (others - leading + 2).max(1) as i32;
    [
        DyadicBound::new(BigInt::from(-1), exponent),
        DyadicBound::new(BigInt::from(1), exponent),
    ]
}
/// Signs of the equation just left and right of an isolated root.
pub(super) fn side_signs(polynomial: &IntegerPolynomial, bracket: &RootBracket) -> [Sign; 2] {
    let (lower, upper) = bracket.bounds();
    if lower == upper {
        return [left_sign(polynomial, lower), polynomial.right_sign(lower)];
    }
    // Nonexact bracket ends are not roots and enclose no other root.
    [polynomial.sign_at(lower), polynomial.sign_at(upper)]
}
/// Transition from one-sided signs; a missing side is a bounded target end.
pub(super) fn transition(sides: [Option<Sign>; 2]) -> Result<SketchContactTransition, Cause> {
    match sides {
        [None, _] | [_, None] => Ok(SketchContactTransition::TargetEnd),
        [Some(Sign::NoSign), _] | [_, Some(Sign::NoSign)] => Err(Cause::Invariant(
            "contact side sign vanished outside an overlap",
        )),
        [Some(left), Some(right)] if left == right => Ok(SketchContactTransition::Touching),
        [Some(_), Some(_)] => Ok(SketchContactTransition::Crossing),
    }
}
