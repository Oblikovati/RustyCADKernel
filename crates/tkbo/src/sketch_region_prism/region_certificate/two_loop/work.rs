// SPDX-License-Identifier: GPL-2.0-only
//! Checked aggregate units precede retained operand allocation; progress stays operation-local.
use super::{failure::*, request::*};
pub(super) type TwoLoopCheck<'a, T> = Result<T, Box<TwoLoopRegionFailure<'a>>>;
pub(super) struct TwoLoopWork<'p> {
    pub operation: u64,
    pub requested: TwoLoopRegionOptions,
    pub stage: TwoLoopRegionStage,
    pub retained_units: usize,
    pub winding_bisections: u16,
    progress: &'p mut dyn TwoLoopRegionProgress,
}
impl<'p> TwoLoopWork<'p> {
    pub fn new(
        operation: u64,
        requested: TwoLoopRegionOptions,
        progress: &'p mut dyn TwoLoopRegionProgress,
    ) -> Self {
        Self {
            operation,
            requested,
            stage: TwoLoopRegionStage::Admission,
            retained_units: 0,
            winding_bisections: 0,
            progress,
        }
    }
    pub fn failure<'a>(&self, cause: TwoLoopRegionCause) -> Box<TwoLoopRegionFailure<'a>> {
        Box::new(TwoLoopRegionFailure {
            operation: self.operation,
            stage: self.stage,
            requested: self.requested,
            retained_units: self.retained_units,
            entities: match &cause {
                TwoLoopRegionCause::WindingLocation { source, .. } => {
                    vec![TwoLoopEntity::Source(*source)]
                }
                _ => Vec::new(),
            },
            cause,
            local: None,
        })
    }
    pub fn poll<'a>(&mut self, stage: TwoLoopRegionStage) -> TwoLoopCheck<'a, ()> {
        self.stage = stage;
        let cancelled = self
            .progress
            .checkpoint(stage)
            .map_err(|cause| self.failure(TwoLoopRegionCause::Native(cause)))?;
        if cancelled {
            return Err(self.failure(TwoLoopRegionCause::Cancelled));
        }
        Ok(())
    }
    pub fn units<'a>(&mut self, operand: &'static str, count: usize) -> TwoLoopCheck<'a, ()> {
        let measured = self
            .retained_units
            .checked_add(count)
            .ok_or_else(|| self.failure(TwoLoopRegionCause::Overflow { operand }))?;
        let requested = self.requested.maximum_retained_units;
        if measured > requested {
            return Err(self.failure(TwoLoopRegionCause::Limit {
                operand,
                requested,
                measured,
            }));
        }
        self.retained_units = measured;
        Ok(())
    }
    pub fn count<'a>(
        &self,
        hypothesis: &'static str,
        actual: usize,
        expected: usize,
    ) -> TwoLoopCheck<'a, ()> {
        if actual != expected {
            return Err(self.failure(TwoLoopRegionCause::Count {
                hypothesis,
                actual,
                expected,
            }));
        }
        Ok(())
    }
    pub fn holds<'a>(&self, hypothesis: &'static str, valid: bool) -> TwoLoopCheck<'a, ()> {
        if !valid {
            return Err(self.failure(TwoLoopRegionCause::Missing { hypothesis }));
        }
        Ok(())
    }
}
