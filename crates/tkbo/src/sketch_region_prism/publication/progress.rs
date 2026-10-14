// SPDX-License-Identifier: GPL-2.0-only
//! One callback sequence; a refused callback remains terminal for downstream adapters.
use rustycad_tkernel::standard::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismPublicationPhase {
    Admission,
    Join,
    Validation,
    History,
    Measures,
    Commit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrismPublicationCheckpoint {
    pub phase: PrismPublicationPhase,
    pub ordinal: usize,
}
pub(crate) trait PrismPublicationProgress: Send {
    /// Stop before accepting the checkpoint; `progress.checkpoint(point)` retains typed failures.
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure>;
}
pub(crate) enum PublicationCallbackCause {
    Cancelled,
    Failed(Failure),
    Exhausted,
}
pub(crate) struct PublicationCallbackStop {
    pub checkpoint: PrismPublicationCheckpoint,
    pub cause: PublicationCallbackCause,
}
pub(super) struct PublicationCallbacks<'a> {
    progress: &'a mut dyn PrismPublicationProgress,
    ordinal: usize,
    halted: bool,
    stopped: Option<PublicationCallbackStop>,
    retain_child_failure: bool,
}
impl<'a> PublicationCallbacks<'a> {
    pub fn new(progress: &'a mut dyn PrismPublicationProgress) -> Self {
        Self {
            progress,
            ordinal: 0,
            halted: false,
            stopped: None,
            retain_child_failure: false,
        }
    }
    /// Select V2 diagnostic retention; `PublicationCallbacks::retaining_children(progress)` keeps actual child headers.
    pub fn retaining_children(progress: &'a mut dyn PrismPublicationProgress) -> Self {
        let mut callbacks = Self::new(progress);
        callbacks.retain_child_failure = true;
        callbacks
    }
    /// Read the operation-local retention decision; `callbacks.retains_child_failure()` never infers missing legacy headers.
    pub fn retains_child_failure(&self) -> bool {
        // ADR0048 V1 remains selectable; only explicit V2 keeps child headers.
        self.retain_child_failure
    }
    pub fn cancelled(&mut self, phase: PrismPublicationPhase) -> bool {
        if self.halted {
            return true;
        }
        let checkpoint = PrismPublicationCheckpoint {
            phase,
            ordinal: self.ordinal,
        };
        if let Err(cause) = self.advance(checkpoint) {
            self.halted = true;
            self.stopped = Some(PublicationCallbackStop { checkpoint, cause });
        }
        self.halted
    }
    pub fn take_stop(&mut self) -> Option<PublicationCallbackStop> {
        self.stopped.take()
    }
    fn advance(
        &mut self,
        checkpoint: PrismPublicationCheckpoint,
    ) -> Result<(), PublicationCallbackCause> {
        self.ordinal = self
            .ordinal
            .checked_add(1)
            .ok_or(PublicationCallbackCause::Exhausted)?;
        match self.progress.checkpoint(checkpoint) {
            Ok(false) => Ok(()),
            Ok(true) => Err(PublicationCallbackCause::Cancelled),
            Err(cause) => Err(PublicationCallbackCause::Failed(cause)),
        }
    }
}

#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;
