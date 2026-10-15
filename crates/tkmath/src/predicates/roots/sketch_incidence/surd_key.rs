// SPDX-License-Identifier: GPL-2.0-only
//! A source key that is a surd ratio `N / D` at an isolated contact (ADR 0080).
//!
//! It is compared lazily: with a rational exactly, by the sign of `N - v D` at
//! the contact; with other keys by refining enclosures, which separates distinct
//! values. Only an unresolved comparison builds the image equation of the key
//! (an elimination resultant), whose common factors then prove a tie exactly.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::coordinate::precision;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::{RationalPolynomial, Ring};
use super::surd_map::{MappedSurd, SurdMap};
use num_bigint::Sign;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SurdKey {
    contact: IsolatedRoot,
    map: SurdMap,
    extra: u32,
}
impl SurdKey {
    /// The key `map` at the contact; `D` must not vanish there.
    pub fn new(contact: IsolatedRoot, map: SurdMap) -> Self {
        Self {
            contact,
            map,
            extra: 0,
        }
    }
    /// Outward enclosure of `N / D` over the contact bracket.
    pub fn enclosure(&self) -> Option<RatioInterval> {
        let [lower, upper] = self.contact.ratio_bounds();
        let parameter = RatioInterval { lower, upper };
        let bits = precision(&self.contact) + self.extra;
        let reciprocal = self
            .map
            .denominator
            .enclose(&parameter, bits)
            .reciprocal()?;
        Some(
            self.map
                .numerator
                .enclose(&parameter, bits)
                .multiply(&reciprocal),
        )
    }
    /// Narrow the contact (or, for an exact contact, the square roots); one work unit.
    pub fn refine(&mut self, work: &mut Work<'_>) -> Result<(), Cause> {
        if self.contact.exact().is_none() {
            return self.contact.bisect(work);
        }
        work.charge()?;
        self.extra += 64;
        Ok(())
    }
    /// Exact order with a rational: the sign of `N - v D` times the sign of `D`.
    pub fn compare_ratio(
        &mut self,
        value: &ExactRatio,
        work: &mut Work<'_>,
    ) -> Result<Ordering, Cause> {
        let shifted = self
            .map
            .numerator
            .add(&self.map.denominator.scale(&value.negated()));
        let sign = shifted.sign_at_root(&mut self.contact, work)?
            * self.map.denominator.sign_at_root(&mut self.contact, work)?;
        Ok(match sign {
            Sign::Minus => Ordering::Less,
            Sign::NoSign => Ordering::Equal,
            Sign::Plus => Ordering::Greater,
        })
    }
    /// The key as an isolated root of its image equation.
    pub fn image(&mut self, work: &mut Work<'_>) -> Result<IsolatedRoot, Cause> {
        let contact = RationalPolynomial::from_integer(&self.contact.equation.polynomial);
        let mapped = MappedSurd::new(&contact, self.map.clone(), work)?;
        mapped.map_root(&mut self.contact, work)
    }
}
