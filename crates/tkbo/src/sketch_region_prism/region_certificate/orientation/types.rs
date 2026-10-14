// SPDX-License-Identifier: GPL-2.0-only
//! ADR0034: owned conditional orientation, without body-validation authority.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PrismOrientationOptions {
    pub version: u32,
    pub embedding: PrismEmbeddingOptions,
}
impl PrismOrientationOptions {
    /// Select orientation behavior 1 without changing embedding limits; `PrismOrientationOptions::v1(options)`.
    pub(crate) fn v1(embedding: PrismEmbeddingOptions) -> Self {
        Self {
            version: 1,
            embedding,
        }
    }
}
pub(crate) struct PrismOrientationEvidence {
    pub(super) requested: PrismOrientationOptions,
    pub(super) embedding: PrismEmbeddingEvidence,
    pub(super) winding: DerivedFromFreshFacedWinding,
    pub(super) jacobian_sign: i8,
    pub(super) faces: [PrismFaceOrientation; 3],
}
/// Algebraic provenance from the fresh classifier, not a separately retained native winding proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DerivedFromFreshFacedWinding {
    pub(super) candidate_facing: i8,
    pub(super) boundary_traversal: i8,
    pub(super) selected_winding: i64,
    pub(super) increasing_parameter: i8,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrismFaceOrientation {
    pub(super) domain: PrismFaceDomain,
    pub(super) actual: Orientation,
    pub(super) expected: Orientation,
    pub(super) trims: Vec<PrismTrimOrientation>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrismTrimOrientation {
    pub(super) authored: PrismAuthoredTrimUse,
    pub(super) actual: Orientation,
    pub(super) expected: Orientation,
}
