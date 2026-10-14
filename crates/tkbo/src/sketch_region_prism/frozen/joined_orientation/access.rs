// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::region_certificate::embedding::PrismEmbeddingOptions;
impl JoinedPrismOrientationCertificate {
    /// Read unchanged explicit behavior; `certificate.requested()`.
    pub(crate) fn requested(&self) -> JoinedPrismOrientationOptions {
        self.requested
    }
    /// Read the join's retained embedding policy; `certificate.embedding_requested()` changes no budget.
    pub(crate) fn embedding_requested(&self) -> PrismEmbeddingOptions {
        self.embedding_requested
    }
    /// Read conditional exact-direction evidence; `certificate.evidence_kind()` grants no body validity.
    pub(crate) fn evidence_kind(&self) -> JoinedOrientationEvidenceKind {
        JoinedOrientationEvidenceKind::ExactConditionalDirections
    }
    /// Borrow complete non-address join bytes; `certificate.binding_bytes()` includes every sidecar.
    pub(crate) fn binding_bytes(&self) -> &[u8] {
        &self.binding
    }
    /// Compare full immutable operands; `certificate.matches_join(join)` uses no hash or live capture.
    pub(crate) fn matches_join(&self, join: &JoinedPrismProof) -> bool {
        self.binding_bytes() == join.replay_bytes()
    }
    /// Read faced-winding derivation; `certificate.winding()` does not claim separately retained native winding.
    pub(crate) fn winding(&self) -> DerivedFromFreshFacedWinding {
        self.winding
    }
    /// Read the existing exact Jacobian sign; `certificate.jacobian_sign()`.
    pub(crate) fn jacobian_sign(&self) -> i8 {
        self.jacobian_sign
    }
    /// Borrow all three named face and six directed trim rows; `certificate.faces()`.
    pub(crate) fn faces(&self) -> &[JoinedFaceDirection] {
        &self.faces
    }
}
impl JoinedFaceDirection {
    /// Borrow existing authored names and complete parent vectors; `face.named()` invents no identity.
    pub(crate) fn named(&self) -> &JoinedPrismFace {
        &self.named
    }
    /// Borrow original disposition multiplicities; `face.parent_dispositions()` does not deduplicate.
    pub(crate) fn parent_dispositions(&self) -> &JoinedParentDispositions {
        &self.parents
    }
    /// Read observed and required cumulative directions; `face.orientations()`.
    pub(crate) fn orientations(&self) -> (Orientation, Orientation) {
        (self.actual, self.expected)
    }
    /// Borrow directed trim conclusions; `face.trims()` keeps both authored seam uses.
    pub(crate) fn trims(&self) -> &[JoinedTrimDirection] {
        &self.trims
    }
}
impl JoinedTrimDirection {
    /// Borrow the authored trim with interval and complete parents; `trim.named()`.
    pub(crate) fn named(&self) -> &JoinedPrismTrim {
        &self.named
    }
    /// Borrow original disposition multiplicities; `trim.parent_dispositions()` does not deduplicate.
    pub(crate) fn parent_dispositions(&self) -> &JoinedParentDispositions {
        &self.parents
    }
    /// Read observed and required cumulative directions; `trim.orientations()`.
    pub(crate) fn orientations(&self) -> (Orientation, Orientation) {
        (self.actual, self.expected)
    }
}
