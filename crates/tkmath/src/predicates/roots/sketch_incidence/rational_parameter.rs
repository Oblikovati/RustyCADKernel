// SPDX-License-Identifier: GPL-2.0-only
//! A target parameter given as a rational function of the contact variable.
//!
//! Its position relative to binary64 values is decided by exact polynomial signs
//! at the contact root (`p - v q` has the sign of `p/q - v` times `sign(q)`), so no
//! image equation is needed; its reported bounds are an outward enclosure over the
//! contact bracket.
use super::super::certificate_arithmetic::RatioInterval;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::coordinate::{Coordinate, precision};
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::report::{SketchLinePosition, outward_ratio};
use num_bigint::Sign;
use std::cmp::Ordering;

/// `numerator / denominator` with a denominator that does not vanish at the contact;
/// surd parts (ADR 0080) are allowed, so an offset's parameter is one too.
pub(super) struct RationalParameter<T = RationalPolynomial> {
    pub numerator: T,
    pub denominator: T,
}
fn ordering(sign: Sign) -> Ordering {
    match sign {
        Sign::Minus => Ordering::Less,
        Sign::NoSign => Ordering::Equal,
        Sign::Plus => Ordering::Greater,
    }
}
impl<T: Coordinate> RationalParameter<T> {
    /// Exact order of the parameter at the contact with a binary64 value.
    pub fn compare(
        &self,
        contact: &mut IsolatedRoot,
        value: f64,
        work: &mut Work<'_>,
    ) -> Result<Ordering, Cause> {
        let denominator = self.denominator.sign_at(contact, work)?;
        if denominator == Sign::NoSign {
            return Err(Cause::Invariant("rational parameter denominator vanished"));
        }
        let shifted = self.numerator.add(&self.denominator.scale(&ratio(-value)));
        Ok(ordering(shifted.sign_at(contact, work)? * denominator))
    }
    /// Exact position relative to two increasing binary64 values.
    pub fn position(
        &self,
        contact: &mut IsolatedRoot,
        range: [f64; 2],
        work: &mut Work<'_>,
    ) -> Result<SketchLinePosition, Cause> {
        Ok(match self.compare(contact, range[0], work)? {
            Ordering::Less => SketchLinePosition::BeforeStart,
            Ordering::Equal => SketchLinePosition::AtStart,
            Ordering::Greater => match self.compare(contact, range[1], work)? {
                Ordering::Less => SketchLinePosition::Interior,
                Ordering::Equal => SketchLinePosition::AtEnd,
                Ordering::Greater => SketchLinePosition::AfterEnd,
            },
        })
    }
    /// Outward binary64 bounds over the contact bracket; None while unbounded.
    pub fn bounds(&self, contact: &IsolatedRoot) -> Option<[f64; 2]> {
        let [lower, upper] = contact.ratio_bounds();
        let parameter = RatioInterval { lower, upper };
        let bits = precision(contact);
        let reciprocal = self.denominator.enclose_at(&parameter, bits).reciprocal()?;
        let value = self
            .numerator
            .enclose_at(&parameter, bits)
            .multiply(&reciprocal);
        Some([
            outward_ratio(&value.lower)[0],
            outward_ratio(&value.upper)[1],
        ])
    }
}
