// SPDX-License-Identifier: GPL-2.0-only
//! Shared owned operands and original typed refusal causes, independent of scalar calculation.
use super::*;
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};

pub(crate) struct CurrentPrismBinding {
    pub region: RegionCertificate,
    pub signed_jacobian: rustycad_tkmath::predicates::roots::DyadicBound,
    pub charts: Vec<BoundPrismChart>,
}
pub(crate) struct BoundPrismChart {
    pub source: PartitionOccurrenceIdentity,
    pub output: PartitionOccurrenceIdentity,
    pub role: PrismImageRole,
    pub parents: Vec<OriginalMemberIdentity>,
    pub captured: CapturedSurfaceChart,
    pub proof: PrismSurfaceProof,
}

// These private failures may retain mutable legacy diagnostics; they are not published bodies.
pub(crate) enum PrismBindingFailure {
    Region(RegionCertificateCause),
    Classification(Box<CandidateCause>),
    Chart(PrismSurfaceCause),
    Body(PrismCompletion),
    Cancelled,
}
impl From<RegionCertificateCause> for PrismBindingFailure {
    fn from(cause: RegionCertificateCause) -> Self {
        // Preserve the exact failed source hypothesis for this private refusal.
        Self::Region(cause)
    }
}
impl From<PrismSurfaceCause> for PrismBindingFailure {
    fn from(cause: PrismSurfaceCause) -> Self {
        // Keep representation and proof-stage context instead of a generic rejection.
        Self::Chart(cause)
    }
}
impl From<PrismCompletion> for PrismBindingFailure {
    fn from(cause: PrismCompletion) -> Self {
        // Legacy body diagnostics remain separate from immutable captured evidence.
        Self::Body(cause)
    }
}
pub(super) type PrismBindingCheck<T = ()> = Result<T, PrismBindingFailure>;
