// SPDX-License-Identifier: GPL-2.0-only
//! Original-equation and domain signs remain exact at each projected algebraic root.
use super::super::RootIsolation;
use super::{
    algebraic,
    operation::{GraphTask, PairOperationFailure},
    polynomial::ExactCoefficientPolynomial,
};
use std::cmp::Ordering;
type MembershipPredicate = fn(Ordering) -> bool;
type MembershipQuery = (ExactCoefficientPolynomial, MembershipPredicate);
impl GraphTask<'_> {
    pub(super) fn membership(
        &self,
        roots: &RootIsolation,
        axis: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Vec<bool>, PairOperationFailure> {
        let mut accepted = vec![true; roots.roots().len()];
        for (query, required) in self.queries(axis, cancelled)? {
            let signs = algebraic::query_signs(
                &self.projection.cleared_originals[axis],
                &query,
                roots,
                self.options.max_degree,
                cancelled,
            )
            .map_err(PairOperationFailure::Sign)?;
            apply_signs(&mut accepted, signs, required);
        }
        Ok(accepted)
    }
    fn queries(
        &self,
        axis: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Vec<MembershipQuery>, PairOperationFailure> {
        let mut queries: Vec<MembershipQuery> = vec![
            (
                self.projection.cleared_originals[1 - axis].clone(),
                |sign| sign == Ordering::Equal,
            ),
            (self.boundary(0, cancelled)?, |sign| sign != Ordering::Less),
            (self.boundary(1, cancelled)?, |sign| {
                sign != Ordering::Greater
            }),
        ];
        self.append_diagonal(&mut queries, cancelled)?;
        Ok(queries)
    }
    fn append_diagonal(
        &self,
        queries: &mut Vec<MembershipQuery>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), PairOperationFailure> {
        if !self.self_domain {
            return Ok(());
        }
        let diagonal =
            algebraic::off_diagonal(&self.projection.alpha, &self.projection.beta, cancelled)
                .map_err(PairOperationFailure::Root)?;
        queries.push((diagonal, |sign| sign == Ordering::Greater));
        Ok(())
    }
    fn boundary(
        &self,
        endpoint: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<ExactCoefficientPolynomial, PairOperationFailure> {
        algebraic::graph_boundary(
            &self.projection.alpha,
            &self.projection.beta,
            self.sources[1].interval[endpoint],
            cancelled,
        )
        .map_err(PairOperationFailure::Root)
    }
}
fn apply_signs(accepted: &mut [bool], signs: Vec<Ordering>, required: MembershipPredicate) {
    for (flag, sign) in accepted.iter_mut().zip(signs) {
        *flag &= required(sign);
    }
}
