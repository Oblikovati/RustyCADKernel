// SPDX-License-Identifier: GPL-2.0-only
//! Re-isolation keeps the original joint polynomial and charges every completed split.
use super::super::{DyadicBound, RootBracket, RootIsolation, polynomial_solve};
use super::polynomial::ExactCoefficientPolynomial;
use super::{operation::GraphTask, operation_failure::PairOperationFailure};

impl GraphTask<'_> {
    pub(super) fn refine_contact_root(
        &self,
        polynomial: &ExactCoefficientPolynomial,
        previous: &RootBracket,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<RootBracket, PairOperationFailure> {
        let refined = self.reisolate_contact(polynomial, previous, cancelled)?;
        linked_root(previous, refined)
    }
    fn reisolate_contact(
        &self,
        polynomial: &ExactCoefficientPolynomial,
        previous: &RootBracket,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<RootIsolation, PairOperationFailure> {
        let (bounds, width) = refinement_bounds(previous);
        let source = polynomial
            .primitive(self.options.max_degree)
            .map_err(PairOperationFailure::Root)?;
        polynomial_solve::solve_polynomial_accounted(
            source,
            bounds,
            width,
            self.budget.effective(self.options),
            cancelled,
            &mut || self.budget.completed_split(),
        )
        .map_err(PairOperationFailure::Root)
    }
}

fn linked_root(
    previous: &RootBracket,
    refined: RootIsolation,
) -> Result<RootBracket, PairOperationFailure> {
    let [next] = refined.roots() else {
        return Err(PairOperationFailure::RefinementInvariant(Box::new(refined)));
    };
    let enclosed = next.bounds().0 >= previous.bounds().0 && next.bounds().1 <= previous.bounds().1;
    if !enclosed || next.multiplicity() != previous.multiplicity() {
        return Err(PairOperationFailure::RefinementInvariant(Box::new(refined)));
    }
    Ok(next.clone())
}

fn refinement_bounds(root: &RootBracket) -> ([DyadicBound; 2], DyadicBound) {
    let bounds = [root.bounds().0.clone(), root.bounds().1.clone()];
    let width = root.width().midpoint(&DyadicBound::from_f64(0.).unwrap());
    (bounds, width)
}
