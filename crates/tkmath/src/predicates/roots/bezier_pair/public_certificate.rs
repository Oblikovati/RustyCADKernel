// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed evidence keeps parameter correspondence distinct from rounded point proximity.
use super::super::{RootBracket, RootIsolation};
use super::ExactPairPolynomial;
use super::operation::{PairOperationProof, PairProofEvidence};
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
    /// Exact rational graph enclosure computed before outward binary64 conversion.
    pub exact_paired_parameter:
        super::public_parameter_enclosure::ExactBezierPairParameterEnclosure<'a>,
    /// Conservative world bounds for the common exact point.
    pub world_bounds: [[f64; 2]; 2],
    /// Deterministic rounded point chosen inside the construction pipeline.
    pub point: [f64; 2],
    /// Outward certified Euclidean construction radius, never a point residual.
    pub achieved_world_error: f64,
}
impl<'a> BezierPairCertificate<'a> {
    pub(super) fn new(proof: &'a PairOperationProof) -> Self {
        let retained = proof;
        // This view cannot detach the evidence from its completed source-bound report.
        Self { proof: retained }
    }
    /// Borrow actual graph evidence only when a graph was computed. Example: `proof.graph().unwrap().candidates()`.
    pub fn graph(&self) -> Option<super::public_graph_certificate::BezierPairGraphCertificate<'a>> {
        let graph = self.proof.graph()?;
        Some(super::public_graph_certificate::BezierPairGraphCertificate::new(graph))
    }
    /// Borrow an exact constant-equation exclusion, without invented graph evidence. Example: `proof.constant_exclusion().unwrap().coordinate_axis()`.
    pub fn constant_exclusion(
        &self,
    ) -> Option<super::public_constant_certificate::BezierPairConstantExclusion<'a>> {
        match &self.proof.evidence {
            PairProofEvidence::ConstantExcluded(proof) => {
                Some(super::public_constant_certificate::BezierPairConstantExclusion::new(proof))
            }
            PairProofEvidence::Graph(_) | PairProofEvidence::CommonCarrier(_) => None,
        }
    }
    /// Borrow ADR 0066 part B common-carrier evidence.
    /// Example: `proof.common_carrier().unwrap().kind()`.
    pub fn common_carrier(&self) -> Option<super::public_carrier::BezierPairCommonCarrier<'a>> {
        match &self.proof.evidence {
            PairProofEvidence::CommonCarrier(evidence) => Some(
                super::public_carrier::BezierPairCommonCarrier::new(evidence),
            ),
            _ => None,
        }
    }
    /// Read whether generated graph-sign admission ran. Example: `proof.sign_stage()`.
    pub fn sign_stage(&self) -> super::public_constant_certificate::BezierPairSignStage {
        match &self.proof.evidence {
            PairProofEvidence::Graph(graph) => {
                super::public_constant_certificate::BezierPairSignStage::Evaluated(graph.signs)
            }
            PairProofEvidence::ConstantExcluded(_) => {
                super::public_constant_certificate::BezierPairSignStage::SkippedConstantExclusion
            }
            PairProofEvidence::CommonCarrier(_) => {
                super::public_constant_certificate::BezierPairSignStage::SkippedCommonCarrier
            }
        }
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
            exact_paired_parameter:
                super::public_parameter_enclosure::ExactBezierPairParameterEnclosure::new(
                    &contact.exact_u,
                ),
            world_bounds: contact.world.bounds,
            point: contact.world.point,
            achieved_world_error: contact.world.achieved_error,
        })
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
