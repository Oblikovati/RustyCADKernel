// SPDX-License-Identifier: GPL-2.0-only
use super::*;

/// Exact incidence over proved parameter domains, never a successful solid operation result.
pub(in crate::sketch_region_prism) struct AnnularTopologyCertificate {
    pub(super) source: Box<[u8]>,
    pub(super) operation: AuthoredPrismId,
    pub(super) root: FrozenPrismKey,
    pub(super) counts: [u32; 7],
}
pub(in crate::sketch_region_prism) enum AnnularTopologyOutcome {
    Certified(Box<AnnularTopologyCertificate>),
    Failed(TopologyFailure),
    Incomplete(TopologyFailure),
    Cancelled {
        operation: AuthoredPrismId,
        checkpoint: TopologyCheckpoint,
    },
}
impl AnnularTopologyCertificate {
    /// Check every joined operand, not just counts; `certificate.binds(joined)`.
    pub(in crate::sketch_region_prism) fn binds(&self, joined: &AnnularFrozenDomain) -> bool {
        self.source.as_ref() == joined.replay_bytes()
    }
    /// Read actual incidence counts; `certificate.counts()` retains vertices through solids.
    pub(in crate::sketch_region_prism) fn counts(&self) -> [u32; 7] {
        self.counts
    }
    /// Read the authored operation; `certificate.operation()` is never a native address.
    pub(in crate::sketch_region_prism) fn operation(&self) -> AuthoredPrismId {
        self.operation
    }
    /// Read the named solid root; `certificate.root()` grants no geometry-validity status.
    pub(in crate::sketch_region_prism) fn root(&self) -> FrozenPrismKey {
        self.root
    }
    /// Borrow the complete certificate input binding; `certificate.source_bytes()`.
    pub(in crate::sketch_region_prism) fn source_bytes(&self) -> &[u8] {
        &self.source
    }
    /// This finite incidence theorem is exact; `certificate.evidence_kind()` is never sampled.
    pub(in crate::sketch_region_prism) fn evidence_kind(&self) -> TopologyEvidenceKind {
        TopologyEvidenceKind::ExactCombinatorial
    }
    /// Integer incidence has no metric error; `certificate.numerical_error()`.
    pub(in crate::sketch_region_prism) fn numerical_error(&self) -> TopologyNumericalError {
        TopologyNumericalError::NotApplicable
    }
}
