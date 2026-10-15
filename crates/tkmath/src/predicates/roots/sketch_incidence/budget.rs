// SPDX-License-Identifier: GPL-2.0-only
//! Operation-local work account: cancellation and the total refinement limit.
use super::super::RootIsolationOptions;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::input::SketchIncidenceOptions;

pub(super) struct Work<'a> {
    used: u32,
    limit: u32,
    world_error: f64,
    isolation: RootIsolationOptions,
    stage: SketchIncidenceStage,
    cancelled: &'a mut dyn FnMut() -> bool,
}
impl<'a> Work<'a> {
    pub fn new(options: SketchIncidenceOptions, cancelled: &'a mut dyn FnMut() -> bool) -> Self {
        Self {
            used: 0,
            limit: options.max_refinements,
            world_error: options.max_world_error,
            isolation: RootIsolationOptions {
                version: 1,
                // Isolation receives an explicit whole-interval width; refinement
                // to the world error limit is charged separately below.
                max_width: 1.,
                max_bisections: options.max_isolation_bisections,
                max_degree: options.max_degree,
            },
            stage: SketchIncidenceStage::Equation,
            cancelled,
        }
    }
    /// Record the stage that owns subsequent failures.
    pub fn enter(&mut self, stage: SketchIncidenceStage) {
        self.stage = stage;
    }
    pub fn stage(&self) -> SketchIncidenceStage {
        self.stage
    }
    /// True when the caller asked to stop.
    pub fn poll(&mut self) -> bool {
        (self.cancelled)()
    }
    /// Account for one bracket refinement.
    pub fn charge(&mut self) -> Result<(), Cause> {
        if self.poll() {
            return Err(Cause::Cancelled);
        }
        if self.used == self.limit {
            return Err(Cause::RefinementExhausted { limit: self.limit });
        }
        self.used += 1;
        Ok(())
    }
    pub fn check(&mut self) -> Result<(), Cause> {
        if self.poll() {
            return Err(Cause::Cancelled);
        }
        Ok(())
    }
    pub fn used(&self) -> u32 {
        self.used
    }
    pub fn isolation(&self) -> RootIsolationOptions {
        self.isolation
    }
    /// Caller world error limit, never widened.
    pub fn world_error(&self) -> f64 {
        self.world_error
    }
    pub fn max_degree(&self) -> u16 {
        self.isolation.max_degree
    }
}
