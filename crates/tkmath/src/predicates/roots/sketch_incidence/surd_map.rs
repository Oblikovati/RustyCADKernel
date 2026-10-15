// SPDX-License-Identifier: GPL-2.0-only
//! Map an isolated contact root `v*` through a surd ratio `K = N / D` (ADR 0080).
//!
//! With `N = n1 + n2 s`, `D = d1 + d2 s` and `s = sqrt(G)`, `K D - N = 0` gives
//! `(K d1 - n1) = -(K d2 - n2) s`, so `F(v, K) = (K d1 - n1)^2 - (K d2 - n2)^2 G`
//! vanishes at `(v*, K*)` and `K*` is a root of `R(K) = Res_v(H(v), F(v, K))`,
//! where `H` is the contact equation. The unique root of `R` inside an exact
//! enclosure of `N / D` over the contact bracket is `K*`: the bracket is narrowed
//! (or, for an exact contact, the square root is refined) until it is unique.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::super::{DyadicBound, RootIsolation, polynomial_solve};
use super::algebraic::{IsolatedRoot, RootEquation};
use super::bivariate::{Bivariate, resultant};
use super::budget::Work;
use super::coordinate::precision;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::surd::Surd;
use num_bigint::{BigInt, Sign};
use std::sync::Arc;

/// The surd ratio `numerator / denominator` of the contact variable.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SurdMap {
    pub numerator: Surd,
    pub denominator: Surd,
}
/// A surd ratio with its exact image equation for one contact equation.
pub(super) struct MappedSurd {
    map: SurdMap,
    image: Arc<RootEquation>,
}
impl MappedSurd {
    /// Image equation of `map` over the roots of `contact`.
    pub fn new(
        contact: &RationalPolynomial,
        map: SurdMap,
        work: &mut Work<'_>,
    ) -> Result<Self, Cause> {
        work.check()?;
        let degree = contact.degree().ok_or(Cause::DegenerateResultant)?;
        let first = Bivariate(
            contact
                .0
                .iter()
                .map(|value| RationalPolynomial::constant(value.clone()))
                .collect(),
        );
        let second = image_bivariate(&map);
        let formal = second.0.len().saturating_sub(1);
        let image = resultant(&first, [degree, formal], &second);
        let polynomial = image
            .to_integer(work.max_degree())?
            .ok_or(Cause::DegenerateResultant)?;
        Ok(Self {
            map,
            image: RootEquation::new(polynomial, work)?,
        })
    }
    /// Certified image root of an isolated contact root; `D` must not vanish there.
    pub fn map_root(
        &self,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<IsolatedRoot, Cause> {
        if self.map.denominator.sign_at_root(contact, work)? == Sign::NoSign {
            return Err(Cause::Invariant(
                "surd map denominator vanished at a contact",
            ));
        }
        let mut extra = 0;
        loop {
            work.check()?;
            let bits = precision(contact) + extra;
            if let Some(root) = self.unique_candidate(contact, bits, work)? {
                return Ok(root);
            }
            if contact.exact().is_some() {
                work.charge()?;
                extra += 64;
            } else {
                contact.bisect(work)?;
            }
        }
    }
    // The image root inside the enclosure at this precision, once it is unique.
    fn unique_candidate(
        &self,
        contact: &IsolatedRoot,
        bits: u32,
        work: &mut Work<'_>,
    ) -> Result<Option<IsolatedRoot>, Cause> {
        let Some(bounds) = self.enclosure(contact, bits) else {
            return Ok(None);
        };
        match self.isolate(bounds, work)?.roots() {
            [only] => Ok(Some(IsolatedRoot::from_bracket(&self.image, only))),
            [] => Err(Cause::Invariant("surd image enclosure missed its root")),
            _ => Ok(None),
        }
    }
    fn enclosure(&self, contact: &IsolatedRoot, bits: u32) -> Option<[DyadicBound; 2]> {
        let [lower, upper] = contact.ratio_bounds();
        let parameter = RatioInterval { lower, upper };
        let reciprocal = self
            .map
            .denominator
            .enclose(&parameter, bits)
            .reciprocal()?;
        let value = self
            .map
            .numerator
            .enclose(&parameter, bits)
            .multiply(&reciprocal);
        Some([
            outward(&value.lower, bits, false),
            outward(&value.upper, bits, true),
        ])
    }
    fn isolate(
        &self,
        bounds: [DyadicBound; 2],
        work: &mut Work<'_>,
    ) -> Result<RootIsolation, Cause> {
        let width = bounds[1].difference(&bounds[0]);
        let options = work.isolation();
        Ok(polynomial_solve::solve_polynomial(
            self.image.polynomial.clone(),
            bounds,
            width,
            options,
            &mut || work.poll(),
        )?)
    }
}
// `F(v, K)` as polynomials in `v` whose coefficients are polynomials in `K`.
fn image_bivariate(map: &SurdMap) -> Bivariate {
    let (n, d) = (&map.numerator, &map.denominator);
    let mixed = n.rational.multiply(&d.rational);
    let radical = n.radical.multiply(&d.radical).multiply(&n.radicand);
    let linear = mixed.add(&radical.scale(&ratio(-1.))).scale(&ratio(-2.));
    let [constant, quadratic] = [n.norm(), d.norm()];
    let length = [&constant, &linear, &quadratic]
        .iter()
        .map(|p| p.0.len())
        .max()
        .unwrap_or(0);
    Bivariate(
        (0..length)
            .map(|power| {
                let at = |polynomial: &RationalPolynomial| polynomial.coefficient(power);
                RationalPolynomial(vec![at(&constant), at(&linear), at(&quadratic)])
            })
            .collect(),
    )
}
/// Dyadic bound of a rational on a `2^-bits` grid, rounded down or up.
fn outward(value: &ExactRatio, bits: u32, upward: bool) -> DyadicBound {
    let scaled = value.numerator() << bits;
    let (quotient, remainder) = (&scaled / value.denominator(), &scaled % value.denominator());
    let floor = if remainder.sign() == Sign::Minus {
        quotient - 1
    } else {
        quotient
    };
    let exact = remainder.sign() == Sign::NoSign;
    let significand = if upward && !exact {
        floor + BigInt::from(1)
    } else {
        floor
    };
    DyadicBound::new(significand, -(bits as i32))
}
