// SPDX-License-Identifier: GPL-2.0-only
//! The successful owned history entry contains no mutable operation owner or process-local identity.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismHistoryPhase {
    Admission,
    Capture,
    Commit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismHistoryFailureClass {
    Incomplete,
    Failed,
}
#[derive(Debug)]
pub(crate) struct PrismHistoryFinding {
    pub operation: AuthoredPrismId,
    pub phase: PrismHistoryPhase,
    pub entity: Option<PrismHistoryUseKey>,
    pub class: PrismHistoryFailureClass,
    pub cause: PrismHistoryCause,
}
/// Legacy owner-bearing diagnostics are retained outside the immutable evidence/finding boundary.
/// Example: inspect `failure.local_diagnostic` while the operation receipt remains alive.
pub(crate) enum PrismHistoryLocalDiagnostic<'a> {
    Join(Box<super::super::proof_join::PrismProofJoinFailure>),
    Partition(&'a crate::sketch_region::PartitionReadFailure),
    Arrangement(&'a crate::sketch_region::ArrangementCompletion),
    Candidate(&'a crate::sketch_region_candidates::CandidateCause),
}
pub(crate) struct PrismHistoryFailure<'a> {
    pub finding: PrismHistoryFinding,
    pub local_diagnostic: Option<PrismHistoryLocalDiagnostic<'a>>,
}
pub(crate) enum PrismHistoryOutcome<'a> {
    Inspected(Box<PrismCompleteHistory>),
    Incomplete(PrismHistoryFailure<'a>),
    Failed(PrismHistoryFailure<'a>),
    Cancelled {
        operation: AuthoredPrismId,
        phase: PrismHistoryPhase,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrismHistorySourceBinding {
    pub operation: AuthoredPrismId,
    pub selected_region: AuthoredPrismId,
    pub complementary_region: AuthoredPrismId,
    pub closure_contact: AuthoredPrismId,
    pub originals: Vec<FrozenOriginalKey>,
}
pub(crate) struct PrismCompleteHistory {
    pub(super) binding: PrismHistorySourceBinding,
    pub(super) requested: PrismHistoryOptions,
    pub(super) captured: capture::CapturedPrismHistory,
    pub(super) joined: Vec<u8>,
    pub(super) replay: Vec<u8>,
    pub(super) deleted: Vec<FrozenOriginalKey>,
}
impl PrismCompleteHistory {
    /// Borrow complete deterministic operands and conclusions; `history.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
    /// Bind this history to every joined operand; `history.binds(&joined)`.
    pub(crate) fn binds(&self, joined: &super::super::proof_join::JoinedPrismProof) -> bool {
        self.joined == joined.replay_bytes()
    }
    /// Borrow explicit operation/source/contact identities; `history.source_binding()`.
    pub(crate) fn source_binding(&self) -> &PrismHistorySourceBinding {
        &self.binding
    }
    /// Requested and effective bounds are identical; `history.requested()`.
    pub(crate) fn requested(&self) -> PrismHistoryOptions {
        self.requested
    }
    /// Borrow the exhaustive original use continuations; `history.continued()`.
    pub(crate) fn continued(&self) -> &[PrismOriginalContinuation] {
        &self.captured.continued
    }
    /// Certified empty only after all original continuations; `assert!(history.deleted_originals().is_empty())`.
    pub(crate) fn deleted_originals(&self) -> &[FrozenOriginalKey] {
        &self.deleted
    }
    /// Borrow every intermediate and complementary disposition; `history.intermediate()`.
    pub(crate) fn intermediate(&self) -> &PrismIntermediateHistory {
        &self.captured.intermediate
    }
    /// Borrow exhaustive sweep answers, including false/empty; `history.sweep_queries()`.
    pub(crate) fn sweep_queries(&self) -> &[PrismCompleteSweepQuery] {
        &self.captured.sweep
    }
    /// Borrow the native partition metadata without temporary indices; `history.partition()`.
    pub(crate) fn partition(&self) -> &PrismPartitionMetadata {
        &self.captured.partition
    }
    /// Borrow the existing copy/sweep proof; `history.lineage()`.
    pub(crate) fn lineage(&self) -> &super::super::lineage::PrismLineageInspection {
        &self.captured.lineage
    }
    /// History alone never validates a body; `assert!(!history.is_valid_body())`.
    pub(crate) fn is_valid_body(&self) -> bool {
        false
    }
}
