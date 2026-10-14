// SPDX-License-Identifier: GPL-2.0-only
//! Map an isolated contact root `v*` through a rational function `P/Q` to the
//! unique root of the image equation `R(T) = Res_v(N, P - T Q)` that equals
//! `P(v*)/Q(v*)`. The contact bracket is narrowed until the exact enclosure of
//! `P/Q` over it holds exactly one root of `R`; an exact dyadic `v*` selects the
//! root equal to the exact rational image instead.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval, rounding};
use super::super::{DyadicBound, RootBracket, RootIsolation, polynomial_solve};
use super::algebraic::{IsolatedRoot, RootEquation};
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::rational_polynomial::RationalPolynomial;
use super::resultant::image_equation;
use super::root_sign::sign_at_root;
use num_bigint::Sign;
use std::sync::Arc;

/// The rational function `numerator / denominator` of the contact variable.
#[derive(Clone, Debug)]
pub(super) struct RationalMap {
    pub numerator: RationalPolynomial,
    pub denominator: RationalPolynomial,
}
/// A rational map with its exact image equation for one contact equation.
pub(super) struct MappedEquation {
    map: RationalMap,
    image: Arc<RootEquation>,
}
impl RationalMap {
    /// Exact value at a rational contact parameter (nonzero denominator).
    pub fn value_at(&self, parameter: &ExactRatio) -> ExactRatio {
        let denominator = self.denominator.evaluate(parameter);
        self.numerator.evaluate(parameter).divide(&denominator)
    }
    /// Outward dyadic enclosure over a bracket, when the denominator excludes zero.
    fn enclosure(&self, root: &IsolatedRoot) -> Option<[DyadicBound; 2]> {
        let [lower, upper] = root.ratio_bounds();
        let parameter = RatioInterval { lower, upper };
        let reciprocal = self.denominator.enclose(&parameter).reciprocal()?;
        let quotient = self.numerator.enclose(&parameter).multiply(&reciprocal);
        let lower = rounding::outward_binary64(&quotient.lower)?[0];
        let upper = rounding::outward_binary64(&quotient.upper)?[1];
        Some([lower, upper].map(dyadic))
    }
}
impl MappedEquation {
    /// Image equation of `map` over the roots of `contact`.
    pub fn new(
        contact: &RationalPolynomial,
        map: RationalMap,
        work: &mut Work<'_>,
    ) -> Result<Self, Cause> {
        work.check()?;
        let image = image_equation(contact, &map.numerator, &map.denominator)
            .ok_or(Cause::DegenerateResultant)?;
        let polynomial = image
            .to_integer(work.max_degree())?
            .ok_or(Cause::DegenerateResultant)?;
        Ok(Self {
            map,
            image: RootEquation::new(polynomial, work)?,
        })
    }
    /// Certified image root of an isolated contact root; the denominator must not vanish.
    pub fn map_root(
        &self,
        contact: &mut IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<IsolatedRoot, Cause> {
        if sign_at_root(contact, &self.map.denominator, work)? == Sign::NoSign {
            return Err(Cause::Invariant(
                "rational map denominator vanished at a contact",
            ));
        }
        loop {
            work.check()?;
            if let Some(bounds) = self.map.enclosure(contact) {
                let candidates = self.isolate_image(bounds, work)?;
                if let Some(root) = self.unique_candidate(&candidates, contact, work)? {
                    return Ok(root);
                }
            } else if contact.exact().is_some() {
                return Err(Cause::ConstructionResolution {
                    requested: work.world_error(),
                    achieved: None,
                });
            }
            contact.bisect(work)?;
        }
    }
    fn unique_candidate(
        &self,
        candidates: &RootIsolation,
        contact: &IsolatedRoot,
        work: &mut Work<'_>,
    ) -> Result<Option<IsolatedRoot>, Cause> {
        let roots = candidates.roots();
        if roots.len() == 1 {
            return Ok(Some(IsolatedRoot::from_bracket(&self.image, &roots[0])));
        }
        if roots.is_empty() {
            return Err(Cause::Invariant(
                "image enclosure missed its resultant root",
            ));
        }
        match contact.exact() {
            Some(exact) => self.exact_candidate(roots, exact, work).map(Some),
            None => Ok(None),
        }
    }
    // An exact contact parameter has an exact rational image; select its root.
    fn exact_candidate(
        &self,
        roots: &[RootBracket],
        exact: &DyadicBound,
        work: &mut Work<'_>,
    ) -> Result<IsolatedRoot, Cause> {
        let value = self.map.value_at(&ExactRatio::from_dyadic(exact));
        for bracket in roots {
            let mut candidate = IsolatedRoot::from_bracket(&self.image, bracket);
            if candidate.compare_ratio(&value, work)?.is_eq() {
                return Ok(candidate);
            }
        }
        Err(Cause::Invariant(
            "exact image value is not a resultant root",
        ))
    }
    fn isolate_image(
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
pub(super) fn dyadic(value: f64) -> DyadicBound {
    DyadicBound::from_f64(value).expect("finite outward bound")
}
