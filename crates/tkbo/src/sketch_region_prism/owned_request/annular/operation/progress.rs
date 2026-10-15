// SPDX-License-Identifier: GPL-2.0-only
//! One caller progress observes every child stage; the first stop is recorded with its exact
//! stage and ordinal, and every later child poll stays stopped.
use super::*;
use crate::sketch_region_prism::publication::{
    PrismPublicationCheckpoint, PrismPublicationProgress,
};
use crate::sketch_region_prism::region_certificate::annular_binding::{
    AnnularBindingProgress, AnnularBindingStage,
};
use crate::sketch_region_prism::region_certificate::two_loop::{
    TwoLoopRegionProgress, TwoLoopRegionStage,
};

/// The one-call operation's stages, in execution order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::sketch_region_prism) enum AnnularOperationStage {
    Preparation,
    Region,
    Binding,
    Naming,
    Embedding,
    Publication,
    Volume,
    Body,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularOperationCheckpoint {
    pub stage: AnnularOperationStage,
    pub ordinal: usize,
}
/// Caller progress; `Ok(true)` cancels, `Err` fails, and both stop every later child poll.
pub(in crate::sketch_region_prism) trait AnnularOperationProgress:
    Send
{
    fn checkpoint(&mut self, point: AnnularOperationCheckpoint) -> Result<bool, Failure>;
}
/// The recorded stop: exact checkpoint and the owned caller cause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularOperationStop {
    pub checkpoint: AnnularOperationCheckpoint,
    pub cause: OwnedDiagnosticCallbackCause,
}
pub(super) struct StageProgress<'p> {
    progress: &'p mut dyn AnnularOperationProgress,
    pub stage: AnnularOperationStage,
    ordinal: usize,
    pub stop: Option<AnnularOperationStop>,
}
impl<'p> StageProgress<'p> {
    pub(super) fn new(progress: &'p mut dyn AnnularOperationProgress) -> Self {
        Self {
            progress,
            stage: AnnularOperationStage::Preparation,
            ordinal: 0,
            stop: None,
        }
    }
    /// Forward one child poll; returns `Ok(true)` once stopped without calling the caller again.
    pub(super) fn forward(&mut self) -> Result<bool, Failure> {
        if self.stop.is_some() {
            return Ok(true);
        }
        let checkpoint = AnnularOperationCheckpoint {
            stage: self.stage,
            ordinal: self.ordinal,
        };
        let Some(next) = self.ordinal.checked_add(1) else {
            return Ok(self.record(checkpoint, OwnedDiagnosticCallbackCause::Exhausted));
        };
        self.ordinal = next;
        match self.progress.checkpoint(checkpoint) {
            Ok(false) => Ok(false),
            Ok(true) => Ok(self.record(checkpoint, OwnedDiagnosticCallbackCause::Cancelled)),
            Err(failure) => {
                self.record(
                    checkpoint,
                    OwnedDiagnosticCallbackCause::Failed(failure.clone()),
                );
                Err(failure)
            }
        }
    }
    /// Boolean form for cancel closures; a caller failure is recorded and cancels.
    pub(super) fn poll(&mut self) -> bool {
        self.forward().unwrap_or(true)
    }
    fn record(
        &mut self,
        checkpoint: AnnularOperationCheckpoint,
        cause: OwnedDiagnosticCallbackCause,
    ) -> bool {
        self.stop = Some(AnnularOperationStop { checkpoint, cause });
        true
    }
}
impl OwnedPrismProgress for StageProgress<'_> {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        self.forward()
    }
}
impl TwoLoopRegionProgress for StageProgress<'_> {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        self.forward()
    }
}
impl AnnularBindingProgress for StageProgress<'_> {
    fn checkpoint(&mut self, _: AnnularBindingStage) -> Result<bool, Failure> {
        self.forward()
    }
}
impl PrismPublicationProgress for StageProgress<'_> {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.forward()
    }
}
