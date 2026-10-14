// SPDX-License-Identifier: GPL-2.0-only
//! Only the concrete sealed join issuer can construct successful annular evidence.
use super::*;
use crate::sketch_region_prism::exact_tolerance::StoredToleranceCap;
#[derive(Default)]
pub(in crate::sketch_region_prism) struct AnnularConformanceEvidence {
    pub body: JoinedConformanceEvidence,
    pub supports: Vec<AnnularSupportEvidence>,
    pub closures: Vec<AnnularClosureEvidence>,
    pub source_reports: Vec<AnnularSourceReport>,
}
pub(in crate::sketch_region_prism) struct AnnularPrismConformance {
    pub(super) binding: Vec<u8>,
    pub(super) limits: FrozenPrismLimits,
    pub(super) evidence: AnnularConformanceEvidence,
}
pub(in crate::sketch_region_prism) struct AnnularPrismTolerance {
    pub(super) binding: Vec<u8>,
    pub(super) conformance: Vec<u8>,
    pub(super) limits: FrozenPrismLimits,
    pub(super) cap: StoredToleranceCap,
    pub(super) metadata: Vec<JoinedToleranceMetadata>,
}
pub(in crate::sketch_region_prism) trait AnnularConformanceProgress:
    Send
{
    /// Cancel immutable proof traversal; `progress.cancelled(JoinedConformancePhase::Endpoints)`.
    fn cancelled(&mut self, phase: JoinedConformancePhase) -> bool;
}
pub(in crate::sketch_region_prism) trait AnnularToleranceProgress:
    Send
{
    /// Cancel metadata observation; `progress.cancelled(JoinedTolerancePhase::Metadata)`.
    fn cancelled(&mut self, phase: JoinedTolerancePhase) -> bool;
}
