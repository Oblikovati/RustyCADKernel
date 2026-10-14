// SPDX-License-Identifier: GPL-2.0-only
use super::super::{PrismCompletion, PrismPhase, PrismProgress, copying::mapped};
use rustycad_tkernel::standard::Failure;

pub(super) struct ChartProgress<'a> {
    pub observer: &'a mut dyn PrismProgress,
    pub failure: Option<Failure>,
    pub stopped: bool,
}
impl<'a> ChartProgress<'a> {
    pub(super) fn new(observer: &'a mut dyn PrismProgress) -> Self {
        // Cancellation state belongs to this inspection, never to a reusable observer.
        Self {
            observer,
            failure: None,
            stopped: false,
        }
    }
    pub(super) fn poll(&mut self) -> bool {
        match self.observer.checkpoint(PrismPhase::History) {
            Ok(stop) => self.stopped |= stop,
            Err(failure) => self.failure = Some(failure),
        }
        self.stopped || self.failure.is_some()
    }
    pub(super) fn completion(self) -> Result<(), PrismCompletion> {
        if let Some(failure) = self.failure {
            return Err(PrismCompletion::Failed(mapped(
                PrismPhase::History,
                failure,
            )));
        }
        if self.stopped {
            return Err(PrismCompletion::Cancelled(PrismPhase::History));
        }
        Ok(())
    }
}
