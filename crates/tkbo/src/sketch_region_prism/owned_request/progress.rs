// SPDX-License-Identifier: GPL-2.0-only
//! One terminal callback state preserves typed callback errors alongside child diagnostic receipts.
use crate::sketch_region::ArrangementPhase;
use crate::sketch_region_candidates::CandidatePhase;
use crate::sketch_region_contact_transport::ContactTransportPhase;
use crate::sketch_region_prism::{PrismPhase, PrismProgress};
use rustycad_tkernel::standard::Failure;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OwnedPrismPhase {
    Admission,
    Construction,
    SourceCapture,
    Census,
    Partition(ArrangementPhase),
    Candidate(CandidatePhase),
    Contact(ContactTransportPhase),
    Prism(PrismPhase),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OwnedPrismCheckpoint {
    pub phase: OwnedPrismPhase,
    pub ordinal: usize,
}
pub(crate) trait OwnedPrismProgress: Send {
    /// Poll actual bounded work; example: `progress.checkpoint(point)`; true cancels.
    fn checkpoint(&mut self, point: OwnedPrismCheckpoint) -> Result<bool, Failure>;
}
pub(crate) enum OwnedPrismCallbackCause {
    Cancelled,
    Failed(Failure),
    Exhausted,
}
pub(crate) struct OwnedPrismCallbackStop {
    pub checkpoint: OwnedPrismCheckpoint,
    pub cause: OwnedPrismCallbackCause,
}
pub(super) struct OwnedPrismCallbacks<'a> {
    progress: &'a mut dyn OwnedPrismProgress,
    ordinal: usize,
    halted: bool,
    pub stop: Option<OwnedPrismCallbackStop>,
}
impl<'a> OwnedPrismCallbacks<'a> {
    pub(super) fn new(progress: &'a mut dyn OwnedPrismProgress) -> Self {
        Self {
            progress,
            ordinal: 0,
            halted: false,
            stop: None,
        }
    }
    pub(super) fn poll(&mut self, phase: OwnedPrismPhase) -> bool {
        if self.halted {
            return true;
        }
        let checkpoint = OwnedPrismCheckpoint {
            phase,
            ordinal: self.ordinal,
        };
        if let Err(cause) = self.advance(checkpoint) {
            self.halted = true;
            self.stop = Some(OwnedPrismCallbackStop { checkpoint, cause });
        }
        self.halted
    }
    fn advance(&mut self, checkpoint: OwnedPrismCheckpoint) -> Result<(), OwnedPrismCallbackCause> {
        self.ordinal = self
            .ordinal
            .checked_add(1)
            .ok_or(OwnedPrismCallbackCause::Exhausted)?;
        match self.progress.checkpoint(checkpoint) {
            Ok(false) => Ok(()),
            Ok(true) => Err(OwnedPrismCallbackCause::Cancelled),
            Err(cause) => Err(OwnedPrismCallbackCause::Failed(cause)),
        }
    }
}
impl PrismProgress for OwnedPrismCallbacks<'_> {
    fn checkpoint(&mut self, phase: PrismPhase) -> Result<bool, Failure> {
        // A typed outer callback error survives separately; child cancellation remains unchanged.
        Ok(self.poll(OwnedPrismPhase::Prism(phase)))
    }
}
