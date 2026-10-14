// SPDX-License-Identifier: GPL-2.0-only
//! The owned V2 history holds immutable values and full replay; there is no raw success constructor.
use super::candidates::{AnnularCandidateCopy, AnnularCandidateHistory};
use super::capture::{AnnularScopeRoster, CapturedAnnularHistory};
use super::continuation::{AnnularOriginalContinuation, AnnularWireRetirement};
use super::sources::AnnularSourceHistory;
use super::sweep_parents::AnnularCapturedSweepParents;
use super::traces::AnnularGeneratedTrace;
use super::*;

/// One authored source and its original directed-use keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnnularHistorySource {
    pub source: RegionSourceId,
    pub originals: Vec<FrozenOriginalKey>,
}
/// Explicit operation, region, candidate and source identities bound by this history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnnularHistorySourceBinding {
    pub operation: AuthoredPrismId,
    pub selected_region: AuthoredPrismId,
    pub selected_candidate: u64,
    pub rejected_candidates: Vec<u64>,
    pub sources: Vec<AnnularHistorySource>,
}
/// Owned finding plus operation-local carriers that cannot cross the ownership boundary.
/// Example: inspect `failure.join` while the borrowed request remains alive.
pub(crate) struct AnnularHistoryFailure<'a> {
    pub history: PrismHistoryFailure<'a>,
    pub join: Option<Box<super::super::annular_join::AnnularFrozenFailure<'a>>>,
}
impl<'a> From<PrismHistoryFailure<'a>> for AnnularHistoryFailure<'a> {
    fn from(history: PrismHistoryFailure<'a>) -> Self {
        Self {
            history,
            join: None,
        }
    }
}
impl AnnularHistoryFailure<'_> {
    /// Borrow the owned immutable finding; `failure.finding().class`.
    pub(crate) fn finding(&self) -> &PrismHistoryFinding {
        &self.history.finding
    }
}
pub(crate) enum AnnularHistoryOutcome<'a> {
    Inspected(Box<AnnularCompleteHistory>),
    Incomplete(AnnularHistoryFailure<'a>),
    Failed(AnnularHistoryFailure<'a>),
    Cancelled {
        operation: AuthoredPrismId,
        phase: PrismHistoryPhase,
    },
}
pub(crate) struct AnnularCompleteHistory {
    pub(super) binding: AnnularHistorySourceBinding,
    pub(super) requested: PrismHistoryOptions,
    pub(super) measured_operand_rows: usize,
    pub(super) captured: CapturedAnnularHistory,
    pub(super) joined: Vec<u8>,
    pub(super) replay: Vec<u8>,
    pub(super) deleted: Vec<FrozenOriginalKey>,
}
impl AnnularCompleteHistory {
    /// Borrow complete deterministic operands and conclusions; `history.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
    /// Bind this history to every joined operand; `history.binds(&joined)`.
    pub(crate) fn binds(&self, joined: &AnnularFrozenDomain) -> bool {
        self.joined == joined.replay_bytes()
    }
    /// Counted stored operand units, not execution work; `history.measured_operand_rows()`.
    pub(crate) fn measured_operand_rows(&self) -> usize {
        self.measured_operand_rows
    }
    /// Requested and effective bounds are identical; `history.requested()`.
    pub(crate) fn requested(&self) -> PrismHistoryOptions {
        self.requested
    }
    /// Borrow explicit operation/source/candidate identities; `history.source_binding()`.
    pub(crate) fn source_binding(&self) -> &AnnularHistorySourceBinding {
        &self.binding
    }
    /// Borrow every scope's authored member roster; `history.rosters()`.
    pub(crate) fn rosters(&self) -> &[AnnularScopeRoster] {
        &self.captured.rosters
    }
    /// Borrow every bound historical context; `history.contexts()`.
    pub(crate) fn contexts(&self) -> &[PrismHistoricalContext] {
        &self.captured.contexts
    }
    /// Borrow source-specific copy, partition and closure history; `history.sources()`.
    pub(crate) fn sources(&self) -> &[AnnularSourceHistory] {
        &self.captured.sources
    }
    /// Borrow continuation-derived partition relations; `history.partition_relations()`.
    pub(crate) fn partition_relations(&self) -> &[PrismHistoricalRelation] {
        &self.captured.partition_relations
    }
    /// Borrow candidate copy queries, links and tree; `history.candidate_copy()`.
    pub(crate) fn candidate_copy(&self) -> &AnnularCandidateCopy {
        &self.captured.candidate_copy
    }
    /// Borrow every authored candidate disposition; `history.candidates()`.
    pub(crate) fn candidates(&self) -> &[AnnularCandidateHistory] {
        &self.captured.candidates
    }
    /// Borrow copied-to-generated traces; `history.traces()`.
    pub(crate) fn traces(&self) -> &[AnnularGeneratedTrace] {
        &self.captured.traces
    }
    /// Borrow exhaustive sweep answers, including false/empty; `history.sweep_queries()`.
    pub(crate) fn sweep_queries(&self) -> &[PrismCompleteSweepQuery] {
        &self.captured.sweep
    }
    /// Borrow captured flat parents bound to native rows; `history.captured_parents()`.
    pub(crate) fn captured_parents(&self) -> &[AnnularCapturedSweepParents] {
        &self.captured.captured_parents
    }
    /// Borrow the existing copy/sweep proof; `history.lineage()`.
    pub(crate) fn lineage(&self) -> &super::super::lineage::PrismLineageInspection {
        &self.captured.lineage
    }
    /// Borrow every original continuation; `history.continued()`.
    pub(crate) fn continued(&self) -> &[AnnularOriginalContinuation] {
        &self.captured.continued
    }
    /// Borrow intermediate wire retirements, never original deletions; `history.retired_wires()`.
    pub(crate) fn retired_wires(&self) -> &[AnnularWireRetirement] {
        &self.captured.retired
    }
    /// Certified empty only after all original continuations; `history.deleted_originals()`.
    pub(crate) fn deleted_originals(&self) -> &[FrozenOriginalKey] {
        &self.deleted
    }
    /// History alone never validates a body; `assert!(!history.is_valid_body())`.
    pub(crate) fn is_valid_body(&self) -> bool {
        false
    }
}
