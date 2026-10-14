// SPDX-License-Identifier: GPL-2.0-only
//! Read-only views bind all evidence to the complete joined operand replay.
use super::*;
impl AnnularPrismConformance {
    /// Match the whole sealed operation; `proof.binds(join)`.
    pub(in crate::sketch_region_prism) fn binds(&self, join: &AnnularFrozenDomain) -> bool {
        self.binding == join.replay_bytes()
    }
    /// Read completed evidence, with no body promotion; `proof.evidence()`.
    pub(in crate::sketch_region_prism) fn evidence(&self) -> &AnnularConformanceEvidence {
        &self.evidence
    }
    /// Read untouched requested and effective operation limits; `proof.operation_limits()`.
    pub(in crate::sketch_region_prism) fn operation_limits(&self) -> &FrozenPrismLimits {
        &self.limits
    }
    /// Read this explicit behavior identity; `proof.version()`.
    pub(in crate::sketch_region_prism) fn version(&self) -> u32 {
        2
    }
}
impl AnnularPrismTolerance {
    /// Match the whole sealed operation; `proof.binds(join)`.
    pub(in crate::sketch_region_prism) fn binds(&self, join: &AnnularFrozenDomain) -> bool {
        self.binding == join.replay_bytes()
    }
    /// Borrow every physical metadata observation and directed context; `proof.metadata()`.
    pub(in crate::sketch_region_prism) fn metadata(&self) -> &[JoinedToleranceMetadata] {
        &self.metadata
    }
    /// Read the explicit cap policy; `proof.cap()`.
    pub(in crate::sketch_region_prism) fn cap(&self) -> super::super::StoredToleranceCap {
        self.cap
    }
    /// Read unchanged operation limits; `proof.operation_limits()`.
    pub(in crate::sketch_region_prism) fn operation_limits(&self) -> &FrozenPrismLimits {
        &self.limits
    }
    /// Read this explicit behavior identity; `proof.version()`.
    pub(in crate::sketch_region_prism) fn version(&self) -> u32 {
        2
    }
}
