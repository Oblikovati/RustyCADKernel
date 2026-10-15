// SPDX-License-Identifier: GPL-2.0-only
//! Exact sign of a polynomial at an isolated algebraic root.
//!
//! Zero is proved by a common factor with the root's equation that vanishes inside
//! the isolating bracket (the bracket holds exactly one root of that equation).
//! A nonzero sign is proved by an exact interval enclosure that excludes zero;
//! the bracket is bisected until it does, which terminates because the value is
//! nonzero. Every bisection is charged to the shared refinement account.
use super::super::DyadicBound;
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::super::chain::{SturmChain, root_count};
use super::super::polynomial::IntegerPolynomial;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::RationalPolynomial;
use num_bigint::Sign;
use std::cmp::Ordering;

/// Exact sign of `polynomial` at the root; the root bracket may be narrowed.
pub(super) fn sign_at_root(
    root: &mut IsolatedRoot,
    polynomial: &RationalPolynomial,
    work: &mut Work<'_>,
) -> Result<Sign, Cause> {
    let Some(integer) = polynomial.to_integer(u16::MAX)? else {
        return Ok(Sign::NoSign);
    };
    if let Some(exact) = root.exact() {
        return Ok(integer.sign_at(exact));
    }
    if integer.degree() <= 1 {
        return linear_sign(root, &integer, work);
    }
    if vanishes_at(root, integer, work)? {
        return Ok(Sign::NoSign);
    }
    enclosed_sign(root, polynomial, work)
}
fn linear_sign(
    root: &mut IsolatedRoot,
    polynomial: &IntegerPolynomial,
    work: &mut Work<'_>,
) -> Result<Sign, Cause> {
    let constant = polynomial.0[0].clone();
    let Some(slope) = polynomial.0.get(1).cloned() else {
        return Ok(constant.sign());
    };
    let crossing = ExactRatio::new(-constant, slope.clone());
    let side = root.compare_ratio(&crossing, work)?;
    let sign = match side {
        Ordering::Less => Sign::Minus,
        Ordering::Equal => Sign::NoSign,
        Ordering::Greater => Sign::Plus,
    };
    Ok(if slope.sign() == Sign::Plus {
        sign
    } else {
        -sign
    })
}
// A common factor with a root in the closed bracket vanishes at the isolated root.
fn vanishes_at(
    root: &IsolatedRoot,
    polynomial: IntegerPolynomial,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    let pair = SturmChain::from_pair(root.equation.polynomial.clone(), polynomial, &mut || {
        work.poll()
    })?;
    let Some(common) = pair.repeated_factor() else {
        return Ok(false);
    };
    has_root_in_closed(common, root.lower.clone(), root.upper.clone(), work)
}
pub(super) fn has_root_in_closed(
    polynomial: IntegerPolynomial,
    lower: DyadicBound,
    upper: DyadicBound,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    let chain = SturmChain::new(polynomial, &mut || work.poll())?;
    let low = chain.endpoint(lower, &mut || work.poll())?;
    let high = chain.endpoint(upper, &mut || work.poll())?;
    Ok(low.root || high.root || root_count(&low, &high)? > 0)
}
fn enclosed_sign(
    root: &mut IsolatedRoot,
    polynomial: &RationalPolynomial,
    work: &mut Work<'_>,
) -> Result<Sign, Cause> {
    loop {
        let [lower, upper] = root.ratio_bounds();
        let value = polynomial.enclose(&RatioInterval { lower, upper });
        if value.lower.sign() == Ordering::Greater {
            return Ok(Sign::Plus);
        }
        if value.upper.sign() == Ordering::Less {
            return Ok(Sign::Minus);
        }
        if root.exact().is_some() {
            return Err(Cause::Invariant(
                "exact enclosure of a nonzero value met zero",
            ));
        }
        root.bisect(work)?;
    }
}
