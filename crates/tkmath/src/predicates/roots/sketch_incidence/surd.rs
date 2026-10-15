// SPDX-License-Identifier: GPL-2.0-only
//! Quadratic surds `P + R sqrt(G)` of one contact variable, with `G > 0` on the
//! curve piece that carries them (ADR 0080).
//!
//! Offset points have coordinates of this form (`G` is the squared speed of the
//! basis), so every composed equation, position test and key is one too. The sign
//! of a surd follows from the signs of `P`, `R` and the norm `P^2 - R^2 G`: equal
//! signs decide alone; opposite signs are decided by the norm. Every sign is an
//! exact polynomial sign, at a rational point or at an isolated root.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::{RationalPolynomial, Ring, ratio};
use super::root_sign::sign_at_root;
use num_bigint::{BigInt, Sign};
use std::cmp::Ordering;
use std::sync::Arc;

/// `rational + radical * sqrt(radicand)`; every surd of one piece shares its radicand.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Surd {
    pub rational: RationalPolynomial,
    pub radical: RationalPolynomial,
    pub radicand: Arc<RationalPolynomial>,
}
impl Surd {
    /// A rational polynomial as a surd with a zero radical part.
    pub fn from_rational(rational: RationalPolynomial, radicand: &Arc<RationalPolynomial>) -> Self {
        Self {
            rational,
            radical: RationalPolynomial(Vec::new()),
            radicand: Arc::clone(radicand),
        }
    }
    /// A rational polynomial as a surd with a zero radical part and unit radicand.
    pub fn rational_only(rational: RationalPolynomial) -> Self {
        Self::from_rational(rational, &Arc::new(RationalPolynomial::constant(ratio(1.))))
    }
    /// True when the radical part vanishes identically.
    pub fn is_rational(&self) -> bool {
        self.radical.degree().is_none()
    }
    /// The norm `P^2 - R^2 G`: the product with the conjugate `P - R sqrt(G)`.
    pub fn norm(&self) -> RationalPolynomial {
        let radical = self
            .radical
            .multiply(&self.radical)
            .multiply(&self.radicand);
        self.rational
            .multiply(&self.rational)
            .add(&radical.scale(&ratio(-1.)))
    }
    /// Exact sign at a rational parameter where the radicand is positive.
    pub fn sign_at_ratio(&self, parameter: &ExactRatio) -> Sign {
        let rational = sign(self.rational.evaluate(parameter).sign());
        let radical = sign(self.radical.evaluate(parameter).sign());
        combine(rational, radical, || {
            sign(self.norm().evaluate(parameter).sign())
        })
    }
    /// Exact sign at an isolated root inside the piece; the bracket may narrow.
    pub fn sign_at_root(
        &self,
        root: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<Sign, Cause> {
        let rational = part_sign(root, &self.rational, work)?;
        let radical = part_sign(root, &self.radical, work)?;
        if radical == Sign::NoSign || rational == Sign::NoSign || rational == radical {
            return Ok(combine(rational, radical, || Sign::NoSign));
        }
        Ok(rational * part_sign(root, &self.norm(), work)?)
    }
    /// Outward enclosure over a parameter interval, with `sqrt(G)` to `bits` bits.
    pub fn enclose(&self, parameter: &RatioInterval, bits: u32) -> RatioInterval {
        let root = sqrt_interval(&self.radicand.enclose(parameter), bits);
        let radical = self.radical.enclose(parameter).multiply(&root);
        self.rational.enclose(parameter).add(&radical)
    }
}
/// Bisections of a probe bracket tried before the exact common-factor test.
const SIGN_PROBES: u32 = 24;
// Exact sign of one part at the root. The root's own equation vanishes there; a
// nonzero value is usually separated by a few probe bisections (on a copy, so the
// bracket is unchanged), which avoids a large gcd; otherwise the exact test runs.
fn part_sign(
    root: &mut IsolatedRoot,
    polynomial: &RationalPolynomial,
    work: &mut Work<'_>,
) -> Result<Sign, Cause> {
    if is_root_equation(root, polynomial) {
        return Ok(Sign::NoSign);
    }
    let mut probe = root.clone();
    for _ in 0..SIGN_PROBES {
        let [lower, upper] = probe.ratio_bounds();
        let value = polynomial.enclose(&RatioInterval { lower, upper });
        if value.lower.sign() == Ordering::Greater {
            return Ok(Sign::Plus);
        }
        if value.upper.sign() == Ordering::Less {
            return Ok(Sign::Minus);
        }
        if probe.exact().is_some() {
            break;
        }
        probe.bisect(work)?;
    }
    sign_at_root(root, polynomial, work)
}
// True when the polynomial is a positive or negative multiple of the root's equation.
fn is_root_equation(root: &IsolatedRoot, polynomial: &RationalPolynomial) -> bool {
    let Ok(Some(integer)) = polynomial.to_integer(u16::MAX) else {
        return false;
    };
    let own = &root.equation.polynomial.0;
    integer.0 == *own || integer.0.iter().map(|c| -c).eq(own.iter().cloned())
}
// Signs of `a + b s` with `s > 0`: one zero part or equal signs decide alone.
fn combine(rational: Sign, radical: Sign, norm: impl FnOnce() -> Sign) -> Sign {
    match (rational, radical) {
        (Sign::NoSign, other) | (other, Sign::NoSign) => other,
        (left, right) if left == right => left,
        (left, _) => left * norm(),
    }
}
fn sign(order: Ordering) -> Sign {
    match order {
        Ordering::Less => Sign::Minus,
        Ordering::Equal => Sign::NoSign,
        Ordering::Greater => Sign::Plus,
    }
}
impl Ring for Surd {
    fn add(&self, other: &Self) -> Self {
        Self {
            rational: self.rational.add(&other.rational),
            radical: self.radical.add(&other.radical),
            radicand: Arc::clone(&self.radicand),
        }
    }
    // `(a + b s)(c + d s) = a c + b d G + (a d + b c) s`.
    fn multiply(&self, other: &Self) -> Self {
        let squared = self
            .radical
            .multiply(&other.radical)
            .multiply(&self.radicand);
        let mixed = self.rational.multiply(&other.radical);
        Self {
            rational: self.rational.multiply(&other.rational).add(&squared),
            radical: mixed.add(&self.radical.multiply(&other.rational)),
            radicand: Arc::clone(&self.radicand),
        }
    }
    fn scale(&self, factor: &ExactRatio) -> Self {
        Self {
            rational: self.rational.scale(factor),
            radical: self.radical.scale(factor),
            radicand: Arc::clone(&self.radicand),
        }
    }
}
/// Outward `[sqrt(lower), sqrt(upper)]` on a `2^-bits` grid; negative parts clamp to 0.
pub(super) fn sqrt_interval(value: &RatioInterval, bits: u32) -> RatioInterval {
    RatioInterval {
        lower: sqrt_bound(&value.lower, bits, false),
        upper: sqrt_bound(&value.upper, bits, true),
    }
}
// floor(sqrt(v 4^bits)) / 2^bits <= sqrt(v) <= (that + 1) / 2^bits, where the
// upper bound keeps the floor when `v 4^bits` is an exact integer square (so a
// rational square root on the grid encloses exactly).
fn sqrt_bound(value: &ExactRatio, bits: u32, upward: bool) -> ExactRatio {
    let scale = BigInt::from(1) << bits;
    if value.sign() != Ordering::Greater {
        return ExactRatio::new(BigInt::from(0), scale);
    }
    let numerator = value.numerator() * &scale * &scale;
    let scaled = &numerator / value.denominator();
    let root = scaled.sqrt();
    let exact = &scaled * value.denominator() == numerator && &root * &root == scaled;
    let root = root + BigInt::from(u8::from(upward && !exact));
    ExactRatio::new(root, scale)
}
