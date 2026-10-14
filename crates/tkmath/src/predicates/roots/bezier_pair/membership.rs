// SPDX-License-Identifier: GPL-2.0-only
//! Original-equation and domain signs remain exact at each projected algebraic root.
use super::super::RootIsolation;
use super::{
    algebraic,
    membership_certificate::{MembershipCertificate, MembershipQuery, MembershipRule},
    operation::{GraphTask, PairOperationFailure},
    polynomial::ExactCoefficientPolynomial,
};
impl GraphTask<'_> {
    pub(super) fn membership(
        &self,
        roots: &RootIsolation,
        axis: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<MembershipCertificate, PairOperationFailure> {
        let mut certificate = MembershipCertificate::new(roots.roots().len());
        for query in self.queries(axis, cancelled)? {
            let evaluation = algebraic::query_evaluation(
                &self.projection.cleared_originals[axis],
                &query.polynomial,
                roots,
                self.signs.effective_limit,
                cancelled,
            )
            .map_err(PairOperationFailure::Sign)?;
            certificate.append(query, evaluation);
        }
        Ok(certificate)
    }
    fn queries(
        &self,
        axis: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Vec<MembershipQuery>, PairOperationFailure> {
        let mut queries = self.base_queries(axis, cancelled)?;
        self.append_diagonal(&mut queries, cancelled)?;
        Ok(queries)
    }
    fn base_queries(
        &self,
        axis: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Vec<MembershipQuery>, PairOperationFailure> {
        let other = self.projection.cleared_originals[1 - axis].clone();
        Ok(vec![
            MembershipQuery::new(self.projection.alpha.clone(), MembershipRule::Nonzero),
            MembershipQuery::new(other, MembershipRule::Zero),
            self.boundary_query(0, MembershipRule::Nonnegative, cancelled)?,
            self.boundary_query(1, MembershipRule::Nonpositive, cancelled)?,
        ])
    }
    fn boundary_query(
        &self,
        endpoint: usize,
        rule: MembershipRule,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<MembershipQuery, PairOperationFailure> {
        let polynomial = self.boundary(endpoint, cancelled)?;
        Ok(MembershipQuery::new(polynomial, rule))
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
        queries.push(MembershipQuery {
            polynomial: diagonal,
            rule: MembershipRule::Positive,
        });
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
