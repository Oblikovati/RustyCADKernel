// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl AnnularValidationCertificate {
    /// Report the exact admitted domain; `proof.mode()` never implies generic solid validation.
    pub(in crate::sketch_region_prism) fn mode(&self) -> AnnularValidationMode {
        AnnularValidationMode::ExactTwoBoundaryPolynomialPrism
    }
    /// Match every sealed input; `proof.binds(join)` does not compare only IDs or counts.
    pub(in crate::sketch_region_prism) fn binds(&self, join: &AnnularFrozenDomain) -> bool {
        self.source.as_ref() == join.replay_bytes()
    }
    /// Read unchanged caller policy; `proof.requested()` never widens a cap.
    pub(in crate::sketch_region_prism) fn requested(&self) -> AnnularValidationOptions {
        self.requested
    }
    /// Read explicit current coverage; `proof.trace().status(OrderedPrismStage::Geometry)`.
    pub(in crate::sketch_region_prism) fn trace(&self) -> &OrderedPrismStageTrace {
        &self.trace
    }
    /// Read historical findings separately; `proof.historical()` does not erase older failures.
    pub(in crate::sketch_region_prism) fn historical(
        &self,
    ) -> &super::super::super::coverage::FrozenPrismCoverage {
        &self.historical
    }
    /// Borrow the accepted topology evidence; `proof.topology()` is present only after ordered completion.
    pub(in crate::sketch_region_prism) fn topology(&self) -> &AnnularTopologyCertificate {
        self.proofs
            .topology
            .as_ref()
            .expect("accepted annular topology proof")
    }
    /// Borrow the accepted orientation evidence; `proof.orientation()` is present only after ordered completion.
    pub(in crate::sketch_region_prism) fn orientation(&self) -> &AnnularOrientationCertificate {
        self.proofs
            .orientation
            .as_ref()
            .expect("accepted annular orientation proof")
    }
    /// Borrow the accepted conformance evidence; `proof.conformance()` is present only after ordered completion.
    pub(in crate::sketch_region_prism) fn conformance(&self) -> &AnnularPrismConformance {
        self.proofs
            .conformance
            .as_ref()
            .expect("accepted annular conformance proof")
    }
    /// Borrow the accepted intersection evidence; `proof.intersection()` is present only after ordered completion.
    pub(in crate::sketch_region_prism) fn intersection(&self) -> &AnnularIntersectionCertificate {
        self.proofs
            .intersection
            .as_ref()
            .expect("accepted annular intersection proof")
    }
    /// Borrow the accepted tolerance evidence; `proof.tolerance()` is present only after ordered completion.
    pub(in crate::sketch_region_prism) fn tolerance(&self) -> &AnnularPrismTolerance {
        self.proofs
            .tolerance
            .as_ref()
            .expect("accepted annular tolerance proof")
    }
}
impl AnnularIntersectionCertificate {
    /// Match all joined operands; `proof.binds(join)` includes the complete source and domain history.
    pub(in crate::sketch_region_prism) fn binds(&self, join: &AnnularFrozenDomain) -> bool {
        self.source.as_ref() == join.replay_bytes()
    }
    /// Read four full domains; `proof.domains()` retains both cap loops and side sources.
    pub(in crate::sketch_region_prism) fn domains(&self) -> &[AnnularFaceDomain; 4] {
        &self.domains
    }
    /// Read four diagonal and six pair conclusions; `proof.incidences()` preserves actual trim operands.
    pub(in crate::sketch_region_prism) fn incidences(&self) -> &[AnnularFaceIncidence; 10] {
        &self.incidences
    }
}
