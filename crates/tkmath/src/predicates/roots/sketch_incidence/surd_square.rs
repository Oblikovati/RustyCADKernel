// SPDX-License-Identifier: GPL-2.0-only
//! Surds whose radicand is the square of a rational polynomial (ADR 0080).
//!
//! A line with a Pythagorean direction such as `(3, 4)`, or a curve with a
//! Pythagorean hodograph, has a squared speed `G = S^2`. Then `sqrt(G) = |S|` and
//! `S` keeps one sign on a piece where `G > 0`, so `P + R sqrt(G)` is the rational
//! polynomial `P + sign(S) R S`. The square root is recovered exactly from the
//! top coefficient down and verified by squaring.
use super::super::certificate_arithmetic::ExactRatio;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::surd::Surd;
use std::cmp::Ordering;

/// The rational polynomial equal to `value` on the piece; `G` must be a square.
pub(super) fn rational_form(
    value: &Surd,
    range: Option<[f64; 2]>,
) -> Result<RationalPolynomial, Cause> {
    let root = polynomial_sqrt(&value.radicand).ok_or(Cause::Invariant(
        "a vanishing surd norm without a square radicand",
    ))?;
    let probe = range.map_or(ratio(0.), |[lower, upper]| {
        ratio(lower).add(&ratio(upper)).divide(&ratio(2.))
    });
    let positive = match root.evaluate(&probe).sign() {
        Ordering::Greater => root,
        Ordering::Less => root.scale(&ratio(-1.)),
        Ordering::Equal => return Err(Cause::Invariant("a square radicand vanished on its piece")),
    };
    Ok(value.rational.add(&value.radical.multiply(&positive)))
}
/// Exact `S` with `S^2 = G` and a positive leading coefficient, when it exists.
pub(super) fn polynomial_sqrt(square: &RationalPolynomial) -> Option<RationalPolynomial> {
    let degree = square.degree()?;
    if degree % 2 == 1 {
        return None;
    }
    let root = RationalPolynomial(descending_root(square, degree / 2)?);
    (root
        .multiply(&root)
        .add(&square.scale(&ratio(-1.)))
        .degree()
        .is_none())
    .then_some(root)
}
// Coefficients of `S` from the top: the coefficient of `t^(half + index)` in `S^2`
// is `2 s_half s_index` plus products of coefficients already found.
fn descending_root(square: &RationalPolynomial, half: usize) -> Option<Vec<ExactRatio>> {
    let mut root = vec![ratio(0.); half + 1];
    root[half] = ratio_sqrt(&square.coefficient(2 * half))?;
    let twice_leading = root[half].multiply(&ratio(2.));
    for index in (0..half).rev() {
        let known = (index + 1..half).fold(ratio(0.), |total, other| {
            total.add(&root[other].multiply(&root[half + index - other]))
        });
        let wanted = square.coefficient(half + index).subtract(&known);
        root[index] = wanted.divide(&twice_leading);
    }
    Some(root)
}
/// Exact rational square root, when it exists; example: `ratio_sqrt(&25/4)` is 5/2.
pub(super) fn ratio_sqrt(value: &ExactRatio) -> Option<ExactRatio> {
    if value.sign() == Ordering::Less {
        return None;
    }
    let exact = |integer: &num_bigint::BigInt| {
        let root = integer.sqrt();
        (&root * &root == *integer).then_some(root)
    };
    Some(ExactRatio::new(
        exact(value.numerator())?,
        exact(value.denominator())?,
    ))
}
