// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularMeasureOptions {
    pub version: u32,
    pub cap_area: PolynomialIntegralOptions,
    pub signed_sweep: PolynomialIntegralOptions,
    pub side_area: PolynomialSideOptions,
    pub maximum_side_subdivisions: u32,
    pub total_area: EnclosureSumOptions,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum AnnularFaceMeasure {
    Cap(CertifiedPolynomialPlaneMeasure),
    Side {
        requested: PolynomialSideOptions,
        effective: PolynomialSideOptions,
        report: CertifiedPolynomialSideIntegral,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularMeasuredFace {
    pub named: NamedAnnularFace,
    pub measure: AnnularFaceMeasure,
}
pub(in crate::sketch_region_prism) struct AnnularPrismMeasures {
    pub(super) requested: AnnularMeasureOptions,
    pub(super) binding: Box<[u8]>,
    pub(super) boundaries: [AnnularBoundaryDirection; 2],
    pub(super) faces: Vec<AnnularMeasuredFace>,
    pub(super) signed_sweep: CertifiedPolynomialMeasure,
    pub(super) total: CertifiedEnclosureSum,
    pub(super) side_subdivisions: u32,
    pub(super) replay: Vec<u8>,
}
#[derive(Default, Debug)]
pub(in crate::sketch_region_prism) struct AnnularMeasurePartial {
    pub faces: Vec<AnnularMeasuredFace>,
    pub signed_sweep: Option<CertifiedPolynomialMeasure>,
    pub total: Option<CertifiedEnclosureSum>,
    /// Includes completed subdivisions in a terminal child, not only successful sides.
    pub side_subdivisions: u32,
}
