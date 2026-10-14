// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularValidationOptions {
    pub version: u32,
    pub stored_cap: StoredToleranceCap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularValidationMode {
    ExactTwoBoundaryPolynomialPrism,
}
#[derive(Default)]
pub(in crate::sketch_region_prism) struct AnnularValidationPartial {
    pub topology: Option<Box<AnnularTopologyCertificate>>,
    pub orientation: Option<AnnularOrientationCertificate>,
    pub conformance: Option<AnnularPrismConformance>,
    pub intersection: Option<AnnularIntersectionCertificate>,
    pub tolerance: Option<AnnularPrismTolerance>,
}
pub(in crate::sketch_region_prism) struct AnnularValidationCertificate {
    pub(super) source: Box<[u8]>,
    pub(super) requested: AnnularValidationOptions,
    pub(super) trace: OrderedPrismStageTrace,
    pub(super) proofs: AnnularValidationPartial,
    pub(super) historical: super::super::super::coverage::FrozenPrismCoverage,
}
pub(in crate::sketch_region_prism) struct AnnularIntersectionCertificate {
    pub(super) source: Box<[u8]>,
    pub(super) domains: [AnnularFaceDomain; 4],
    pub(super) incidences: [AnnularFaceIncidence; 10],
    pub(super) embedding: Box<[u8]>,
}
