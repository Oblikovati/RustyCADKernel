// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkernel::standard::Failure;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularValidationCheck {
    Admission,
    Topology(topology::TopologyCheckpoint),
    Orientation { commit: bool },
    Conformance(JoinedConformancePhase),
    Incidence { completed: usize },
    Tolerance(JoinedTolerancePhase),
    Commit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularValidationCheckpoint {
    pub stage: OrderedPrismStage,
    pub check: AnnularValidationCheck,
    pub ordinal: usize,
}
pub(in crate::sketch_region_prism) trait AnnularValidationProgress:
    Send
{
    /// Stop before accepting more evidence; `progress.checkpoint(point)` preserves the actual cause.
    fn checkpoint(&mut self, point: AnnularValidationCheckpoint) -> Result<bool, Failure>;
}
pub(in crate::sketch_region_prism) enum AnnularValidationCallbackCause {
    Cancelled,
    Failed(Failure),
    Exhausted,
}
pub(in crate::sketch_region_prism) struct AnnularValidationCallbackStop {
    pub checkpoint: AnnularValidationCheckpoint,
    pub cause: AnnularValidationCallbackCause,
}
pub(super) struct AnnularValidationContext<'a> {
    pub current: AnnularValidationCheckpoint,
    pub trace: OrderedPrismStageTrace,
    progress: &'a mut dyn AnnularValidationProgress,
    stop: Option<AnnularValidationCallbackStop>,
}
impl<'a> AnnularValidationContext<'a> {
    pub fn new(progress: &'a mut dyn AnnularValidationProgress) -> Self {
        Self {
            current: AnnularValidationCheckpoint {
                stage: OrderedPrismStage::Topology,
                check: AnnularValidationCheck::Admission,
                ordinal: 0,
            },
            trace: OrderedPrismStageTrace::pending(),
            progress,
            stop: None,
        }
    }
    pub fn poll(&mut self, check: AnnularValidationCheck) -> bool {
        if self.stop.is_some() {
            return true;
        }
        self.current.check = check;
        let Some(ordinal) = self.current.ordinal.checked_add(1) else {
            self.stop(AnnularValidationCallbackCause::Exhausted);
            return true;
        };
        self.current.ordinal = ordinal;
        match self.progress.checkpoint(self.current) {
            Ok(false) => false,
            Ok(true) => self.stop(AnnularValidationCallbackCause::Cancelled),
            Err(cause) => self.stop(AnnularValidationCallbackCause::Failed(cause)),
        }
    }
    fn stop(&mut self, cause: AnnularValidationCallbackCause) -> bool {
        self.stop = Some(AnnularValidationCallbackStop {
            checkpoint: self.current,
            cause,
        });
        true
    }
    pub fn accept<T>(&mut self, result: ValidationCheck<T>) -> ValidationCheck<T> {
        let Some(stop) = self.stop.take() else {
            return result;
        };
        let class = callback_class(&stop.cause);
        Err(ValidationStop {
            class,
            cause: AnnularValidationCause::Callback {
                stop: Box::new(stop),
                downstream: result.err().map(Box::new),
            },
        })
    }
    pub fn checkpoint(&mut self, check: AnnularValidationCheck) -> ValidationCheck<()> {
        let stopped = self.poll(check);
        let result = if stopped {
            Err(ValidationStop::cancelled())
        } else {
            Ok(())
        };
        self.accept(result)
    }
    /// Preserve the required stage on refusal; `context.refused(OrderedPrismFailureClass::Incomplete)`.
    pub fn refused(&mut self, class: OrderedPrismFailureClass) {
        self.trace
            .record_prism_stage(self.current.stage, class.into())
            .expect("annular validator refuses only its next required stage");
    }
    pub fn passed(&mut self, stage: OrderedPrismStage) {
        self.trace
            .record_prism_stage(stage, OrderedPrismStageStatus::Passed)
            .expect("annular validator accepts only the next required stage");
        if let Some(next) = self.trace.next() {
            self.current.stage = next;
        }
    }
}
fn callback_class(cause: &AnnularValidationCallbackCause) -> OrderedPrismFailureClass {
    match cause {
        AnnularValidationCallbackCause::Cancelled => OrderedPrismFailureClass::Cancelled,
        AnnularValidationCallbackCause::Failed(_) => OrderedPrismFailureClass::Failed,
        AnnularValidationCallbackCause::Exhausted => OrderedPrismFailureClass::Incomplete,
    }
}
