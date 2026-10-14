// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl PrismOrientationEvidence {
    /// Borrow unchanged requested policy; `evidence.requested()`.
    pub(crate) fn requested(&self) -> PrismOrientationOptions {
        self.requested
    }
    /// Borrow the fresh prerequisite and its full captured operands; `evidence.embedding()`.
    pub(crate) fn embedding(&self) -> &PrismEmbeddingEvidence {
        &self.embedding
    }
    /// Borrow explicitly derived native-parameter orientation; `evidence.winding()`.
    pub(crate) fn winding(&self) -> DerivedFromFreshFacedWinding {
        self.winding
    }
    /// Read the nonzero exact determinant sign; `evidence.jacobian_sign()`.
    pub(crate) fn jacobian_sign(&self) -> i8 {
        self.jacobian_sign
    }
    /// Borrow three role-bound face and six trim conclusions; `evidence.faces()`.
    pub(crate) fn faces(&self) -> &[PrismFaceOrientation; 3] {
        &self.faces
    }
}
impl DerivedFromFreshFacedWinding {
    /// Read candidate-facing, traversal, selected winding and derived native sign; `proof.factors()`.
    pub(crate) fn factors(&self) -> (i8, i8, i64, i8) {
        (
            self.candidate_facing,
            self.boundary_traversal,
            self.selected_winding,
            self.increasing_parameter,
        )
    }
}
impl PrismFaceOrientation {
    /// Borrow authored full domain and parents; `face.domain()`.
    pub(crate) fn domain(&self) -> &PrismFaceDomain {
        &self.domain
    }
    /// Read observed and required cumulative orientations; `face.orientations()`.
    pub(crate) fn orientations(&self) -> (Orientation, Orientation) {
        (self.actual, self.expected)
    }
    /// Borrow every directed trim conclusion, including both seam uses; `face.trims()`.
    pub(crate) fn trims(&self) -> &[PrismTrimOrientation] {
        &self.trims
    }
}
impl PrismTrimOrientation {
    /// Borrow the authored directed use and all parents; `trim.authored()`.
    pub(crate) fn authored(&self) -> &PrismAuthoredTrimUse {
        &self.authored
    }
    /// Read observed and required cumulative orientations; `trim.orientations()`.
    pub(crate) fn orientations(&self) -> (Orientation, Orientation) {
        (self.actual, self.expected)
    }
}
