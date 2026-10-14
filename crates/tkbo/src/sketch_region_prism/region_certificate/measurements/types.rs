// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PrismMeasureOptions {
    pub cap_area: PolynomialIntegralOptions,
    pub signed_volume: PolynomialIntegralOptions,
}
// This owns captured coefficients and operation-local identities only, never Shape/Handle aliases.
pub(crate) struct PrismMeasureInspection {
    pub signed_jacobian: rustycad_tkmath::predicates::roots::DyadicBound,
    pub region: RegionCertificate,
    pub charts: Vec<MeasuredPrismChart>,
    pub measures: PrismMeasureSubset,
    pub pending: [PendingMeasure; 2],
}
pub(crate) struct MeasuredPrismChart {
    pub source: PartitionOccurrenceIdentity,
    pub output: PartitionOccurrenceIdentity,
    pub role: PrismImageRole,
    pub parents: Vec<OriginalMemberIdentity>,
    pub captured: CapturedSurfaceChart,
    pub proof: PrismSurfaceProof,
}
pub(crate) struct PrismMeasureSubset {
    pub first_cap: CertifiedPolynomialPlaneMeasure,
    pub last_cap: CertifiedPolynomialPlaneMeasure,
    pub signed_volume: CertifiedPolynomialMeasure,
    pub extrusion: [f64; 3],
    pub requested: PrismMeasureOptions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PendingMeasure {
    SideArea,
    TotalArea,
}
// Private refusal preserves legacy typed causes; these may carry mutable diagnostic Shapes.
// No failure here is an immutable published operation result or a partially accepted body.
pub(crate) enum PrismMeasureFailure {
    Region(RegionCertificateCause),
    Classification(Box<CandidateCause>),
    Chart(PrismSurfaceCause),
    Body(PrismCompletion),
    Cap {
        role: PrismImageRole,
        cause: PolynomialPlaneError,
    },
    Sweep(PolynomialIntegralError),
    Cancelled,
}
impl From<RegionCertificateCause> for PrismMeasureFailure {
    fn from(cause: RegionCertificateCause) -> Self {
        // Preserve the exact failed source hypothesis for this private refusal.
        Self::Region(cause)
    }
}
impl From<PrismSurfaceCause> for PrismMeasureFailure {
    fn from(cause: PrismSurfaceCause) -> Self {
        // Keep representation and proof-stage context instead of a generic rejection.
        Self::Chart(cause)
    }
}
impl From<PrismCompletion> for PrismMeasureFailure {
    fn from(cause: PrismCompletion) -> Self {
        // Legacy body diagnostics remain separate from immutable scalar evidence.
        Self::Body(cause)
    }
}
pub(super) type MeasureCheck<T = ()> = Result<T, PrismMeasureFailure>;
