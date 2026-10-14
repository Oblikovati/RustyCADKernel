// SPDX-License-Identifier: GPL-2.0-only
use super::{errors::*, *};
use rustycad_tkernel::standard::Failure;

pub(super) struct OrderedWork<'a> {
    pub current: OrderedPrismCheckpoint,
    pub trace: OrderedPrismStageTrace,
    progress: &'a mut dyn OrderedPrismValidationProgress,
    callback_failure: Option<Failure>,
}
impl<'a> OrderedWork<'a> {
    pub fn new(progress: &'a mut dyn OrderedPrismValidationProgress) -> Self {
        Self {
            current: OrderedPrismCheckpoint {
                stage: OrderedPrismStage::Topology,
                check: OrderedPrismCheck::Admission,
                ordinal: 0,
            },
            trace: OrderedPrismStageTrace::pending(),
            progress,
            callback_failure: None,
        }
    }
    pub fn poll(&mut self, check: OrderedPrismCheck) -> bool {
        self.current.check = check;
        self.current.ordinal += 1;
        match self.progress.checkpoint(self.current) {
            Ok(cancelled) => cancelled,
            Err(cause) => {
                self.callback_failure = Some(cause);
                true
            }
        }
    }
    pub fn accept<T>(&mut self, result: OrderedCheck<T>) -> OrderedCheck<T> {
        if let Some(cause) = self.callback_failure.take() {
            return Err(OrderedStop::incomplete(OrderedPrismCause::Callback(
                Box::new(cause),
            )));
        }
        result
    }
    pub fn checkpoint(&mut self, check: OrderedPrismCheck) -> OrderedCheck<()> {
        let result = if self.poll(check) {
            Err(OrderedStop::cancelled())
        } else {
            Ok(())
        };
        self.accept(result)
    }
    pub fn passed(&mut self, stage: OrderedPrismStage) {
        self.trace
            .record_prism_stage(stage, OrderedPrismStageStatus::Passed)
            .expect("internal ordered validator must complete only its next stage");
        if let Some(next) = self.trace.next() {
            self.current.stage = next;
        }
    }
}
