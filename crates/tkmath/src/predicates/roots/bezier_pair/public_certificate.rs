// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed evidence keeps parameter correspondence distinct from rounded point proximity.
use super::super::{RootBracket, RootIsolation, signs::RootSignEvaluation};
use super::ExactPairPolynomial;
use super::{BezierPairEffectiveSigns, BezierPairMembershipRule, operation::PairOperationProof};
/// Read-only view of a genuine completed provider report; no public construction path.
pub struct BezierPairCertificate<'a> {
    proof: &'a PairOperationProof,
}
/// A selected joint root, its paired graph enclosure and certified world construction.
pub struct CertifiedBezierPairContact<'a> {
    /// Stable index into the retained complete candidate isolation, including rejected roots.
    pub candidate_index: usize,
    /// Refined exact bracket containing that same original candidate root.
    pub parameter: &'a RootBracket,
    /// Outward bounds on the other parameter, linked through the exact ideal-row graph.
    pub paired_parameter_bounds: [f64; 2],
    /// Conservative world bounds for the common exact point.
    pub world_bounds: [[f64; 2]; 2],
    /// Deterministic rounded point chosen inside the construction pipeline.
    pub point: [f64; 2],
    /// Outward certified Euclidean construction radius, never a point residual.
    pub achieved_world_error: f64,
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
impl<'a> BezierPairCertificate<'a> {
    pub(super) fn new(proof: &'a PairOperationProof) -> Self {
        let retained = proof;
        // This view cannot detach the evidence from its completed source-bound report.
        Self { proof: retained }
    }
    /// Read the number of fully certified contacts. Example: `proof.contact_count() == 0` is completed empty.
    pub fn contact_count(&self) -> usize {
        let selected = &self.proof.contacts;
        // Rejected projection roots remain separately available in the coverage isolation.
        selected.len()
    }
    /// Read a selected contact by deterministic order. Example: `proof.contact(0).unwrap().candidate_index`.
    pub fn contact(&self, index: usize) -> Option<CertifiedBezierPairContact<'a>> {
        let contact = self.proof.contacts.get(index)?;
        Some(CertifiedBezierPairContact {
            candidate_index: contact.candidate_index,
            parameter: &contact.root,
            paired_parameter_bounds: contact.u_bounds,
            world_bounds: contact.world.bounds,
            point: contact.world.point,
            achieved_world_error: contact.world.achieved_error,
        })
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
    /// Read both actual denominator isolation certificates. Example: `proof.denominators()[0].roots().is_empty()`.
    pub fn denominators(&self) -> [&'a RootIsolation; 2] {
        let sources = &self.proof.source_certificates;
        sources.each_ref().map(|source| &source.denominator)
    }
    /// Read the captured exact homogeneous power coefficients. Example: `proof.homogeneous_sources()[0][2]`.
    pub fn homogeneous_sources(&self) -> [[ExactPairPolynomial<'a>; 3]; 2] {
        self.proof.source_certificates.each_ref().map(|source| {
            source
                .homogeneous
                .components
                .each_ref()
                .map(|p| ExactPairPolynomial::new(&p.0))
        })
    }
    /// Read requested and effective generated-sign admission separately. Example: `proof.effective_signs().effective_limit`.
    pub fn effective_signs(&self) -> BezierPairEffectiveSigns {
        let effective = self.proof.signs;
        // The primary root degree cap is retained in report.policy(), never widened here.
        effective
    }
    /// Read the actual source coefficients used in this completed proof. Example: `proof.captured_curves()[0].controls`.
    pub fn captured_curves(&self) -> [super::BezierPairCurve<'a>; 2] {
        let sources = &self.proof.sources;
        super::public_query::curve_views(sources)
    }
    /// Read successful-stage split work independently of the outer report. Example: `proof.work().completed`.
    pub fn work(&self) -> super::BezierPairWork {
        let completed = self.proof.work;
        // Every stage charges the same execution-owned budget; this is not a replay count.
        completed
    }
}
