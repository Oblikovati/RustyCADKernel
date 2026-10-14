// SPDX-License-Identifier: GPL-2.0-only
//! Graph-only evidence cannot be fabricated for a constant whole-domain exclusion.
use super::super::{RootIsolation, signs::RootSignEvaluation};
use super::operation::GraphProofEvidence;
use super::{BezierPairEffectiveSigns, BezierPairMembershipRule, ExactPairPolynomial};
/// Borrowed actual graph, root, membership and exceptional-fibre proof.
pub struct BezierPairGraphCertificate<'a> {
    proof: &'a GraphProofEvidence,
}
/// Exact ascending coefficients of alpha(t)u+beta(t)=0 and both cleared originals.
pub struct BezierPairProjection<'a> {
    /// Exact ascending coefficient vector for the paired-parameter leading term.
    pub alpha: ExactPairPolynomial<'a>,
    /// Exact ascending coefficient vector for the paired-parameter constant term.
    pub beta: ExactPairPolynomial<'a>,
    /// Both original source equations after exact graph substitution.
    pub cleared_originals: [ExactPairPolynomial<'a>; 2],
}
/// One exact predicate and its aligned source-root signs, including rejected candidates.
pub struct BezierPairMembership<'a> {
    /// Actual query coefficients, retained without numerical approximation.
    pub coefficients: ExactPairPolynomial<'a>,
    /// Required exact predicate at each original candidate root.
    pub rule: BezierPairMembershipRule,
    /// Aligned original candidate roots and exact algebraic signs.
    pub evaluation: &'a RootSignEvaluation,
}
impl<'a> BezierPairGraphCertificate<'a> {
    pub(super) fn new(proof: &'a GraphProofEvidence) -> Self {
        let retained = proof;
        // A graph view always borrows real computed evidence from its owning report.
        Self { proof: retained }
    }
    /// Read the actual original-root polynomial and complete isolation. Example: `proof.candidates().1.roots()`.
    pub fn candidates(&self) -> (ExactPairPolynomial<'a>, &'a RootIsolation) {
        let certificate = &self.proof.candidate_certificate;
        (
            ExactPairPolynomial::new(&certificate.polynomial.0),
            &certificate.isolation,
        )
    }
    /// Read exact original-equation and domain decisions. Example: `proof.membership().count()`.
    pub fn membership(&self) -> impl ExactSizeIterator<Item = BezierPairMembership<'a>> {
        self.proof
            .candidate_certificate
            .membership
            .queries
            .iter()
            .map(|certificate| BezierPairMembership {
                coefficients: ExactPairPolynomial::new(&certificate.query.polynomial.0),
                rule: certificate.query.rule,
                evaluation: &certificate.evaluation,
            })
    }
    /// Read the aligned selected-root mask without losing rejected roots. Example: `proof.accepted_candidates()[0]`.
    pub fn accepted_candidates(&self) -> &'a [bool] {
        let certificate = &self.proof.candidate_certificate;
        &certificate.membership.accepted
    }
    /// Read the actual linear ideal row and substitution into both originals. Example: `proof.projection().alpha`.
    pub fn projection(&self) -> BezierPairProjection<'a> {
        let projection = &self.proof.projection;
        BezierPairProjection {
            alpha: ExactPairPolynomial::new(&projection.alpha.0),
            beta: ExactPairPolynomial::new(&projection.beta.0),
            cleared_originals: projection
                .cleared_originals
                .each_ref()
                .map(|p| ExactPairPolynomial::new(&p.0)),
        }
    }
    /// Read every retained pseudo-remainder leading factor. Example: `proof.leading_factors().count()`.
    pub fn leading_factors(&self) -> impl ExactSizeIterator<Item = ExactPairPolynomial<'a>> {
        let projection = &self.proof.projection;
        projection
            .leading_factors
            .iter()
            .map(|factor| ExactPairPolynomial::new(&factor.0))
    }
    /// Read complete exceptional-fibre exclusion signs. Example: `proof.exceptional_fibers().signs()`.
    pub fn exceptional_fibers(&self) -> &'a RootSignEvaluation {
        let excluded = &self.proof.exceptional_fibers;
        // A common alpha/beta zero would have prevented completion.
        excluded
    }
    /// Read requested and effective generated-sign admission separately. Example: `proof.effective_signs().effective_limit`.
    pub fn effective_signs(&self) -> BezierPairEffectiveSigns {
        let effective = self.proof.signs;
        // The primary root degree cap is retained in report.policy(), never widened here.
        effective
    }
}
