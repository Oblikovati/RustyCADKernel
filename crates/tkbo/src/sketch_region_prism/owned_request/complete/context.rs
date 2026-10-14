// SPDX-License-Identifier: GPL-2.0-only
//! Child ordinals remain provenance; only this context assigns whole-operation order.
use super::super::{OwnedPrismCheckpoint, OwnedPrismPhase, OwnedPrismProgress};
use crate::sketch_region_prism::publication::*;
use rustycad_tkernel::standard::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompletePrismPhase {
    Preparation(OwnedPrismPhase),
    Resolution,
    Publication(PrismPublicationPhase),
    Names,
    Reservations,
    Body,
    Volume,
    Commit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompletePrismCheckpoint {
    pub phase: CompletePrismPhase,
    pub ordinal: usize,
    pub child_ordinal: Option<usize>,
}
pub(crate) trait CompletePrismProgress: Send {
    /// Poll one operation-local sequence; `progress.checkpoint(point)` returns true to cancel.
    fn checkpoint(&mut self, point: CompletePrismCheckpoint) -> Result<bool, Failure>;
}
#[derive(Clone)]
pub(crate) enum CompletePrismCallbackCause {
    Cancelled,
    Failed(Failure),
    Exhausted,
}
#[derive(Clone)]
pub(crate) struct CompletePrismCallbackStop {
    pub checkpoint: CompletePrismCheckpoint,
    pub cause: CompletePrismCallbackCause,
}
pub(super) struct CompletePrismContext<'a> {
    progress: &'a mut dyn CompletePrismProgress,
    ordinal: usize,
    pub stop: Option<CompletePrismCallbackStop>,
}
impl<'a> CompletePrismContext<'a> {
    pub fn new(progress: &'a mut dyn CompletePrismProgress) -> Self {
        Self {
            progress,
            ordinal: 0,
            stop: None,
        }
    }
    pub fn poll(&mut self, phase: CompletePrismPhase, child_ordinal: Option<usize>) -> bool {
        if self.stop.is_some() {
            return true;
        }
        let checkpoint = CompletePrismCheckpoint {
            phase,
            ordinal: self.ordinal,
            child_ordinal,
        };
        if let Err(cause) = self.advance(checkpoint) {
            self.stop = Some(CompletePrismCallbackStop { checkpoint, cause });
        }
        self.stop.is_some()
    }
    fn advance(
        &mut self,
        point: CompletePrismCheckpoint,
    ) -> Result<(), CompletePrismCallbackCause> {
        self.ordinal = self
            .ordinal
            .checked_add(1)
            .ok_or(CompletePrismCallbackCause::Exhausted)?;
        match self.progress.checkpoint(point) {
            Ok(false) => Ok(()),
            Ok(true) => Err(CompletePrismCallbackCause::Cancelled),
            Err(cause) => Err(CompletePrismCallbackCause::Failed(cause)),
        }
    }
    fn child_poll(&mut self, phase: CompletePrismPhase, ordinal: usize) -> Result<bool, Failure> {
        let halted = self.poll(phase, Some(ordinal));
        if let Some(CompletePrismCallbackStop {
            cause: CompletePrismCallbackCause::Failed(cause),
            ..
        }) = &self.stop
        {
            return Err(cause.clone());
        }
        Ok(halted)
    }
}
impl OwnedPrismProgress for CompletePrismContext<'_> {
    fn checkpoint(&mut self, point: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        self.child_poll(CompletePrismPhase::Preparation(point.phase), point.ordinal)
    }
}
impl PrismPublicationProgress for CompletePrismContext<'_> {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.child_poll(CompletePrismPhase::Publication(point.phase), point.ordinal)
    }
}
