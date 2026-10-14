// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::region_certificate::embedding::PrismEmbeddingOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct JoinedPrismOrientationOptions {
    pub version: u32,
}
impl JoinedPrismOrientationOptions {
    /// Choose named orientation behavior 1; `JoinedPrismOrientationOptions::v1()` changes no existing budget.
    pub(crate) fn v1() -> Self {
        Self { version: 1 }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedOrientationEvidenceKind {
    ExactConditionalDirections,
}
pub(crate) struct JoinedPrismOrientationCertificate {
    pub(super) requested: JoinedPrismOrientationOptions,
    pub(super) embedding_requested: PrismEmbeddingOptions,
    pub(super) binding: Box<[u8]>,
    pub(super) winding: DerivedFromFreshFacedWinding,
    pub(super) jacobian_sign: i8,
    pub(super) faces: Vec<JoinedFaceDirection>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct JoinedFaceDirection {
    pub(super) named: JoinedPrismFace,
    pub(super) parents: JoinedParentDispositions,
    pub(super) actual: Orientation,
    pub(super) expected: Orientation,
    pub(super) trims: Vec<JoinedTrimDirection>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct JoinedTrimDirection {
    pub(super) named: JoinedPrismTrim,
    pub(super) parents: JoinedParentDispositions,
    pub(super) actual: Orientation,
    pub(super) expected: Orientation,
}
