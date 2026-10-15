// SPDX-License-Identifier: GPL-2.0-only
//! Stage transitions carry no geometry proof; only the complete operation can publish one.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrderedPrismStage {
    Topology,
    Geometry,
    SelfIntersection,
    Tolerance,
}
impl OrderedPrismStage {
    pub(in crate::sketch_region_prism) const ALL: [Self; 4] = [
        Self::Topology,
        Self::Geometry,
        Self::SelfIntersection,
        Self::Tolerance,
    ];

    pub(super) fn index(self) -> usize {
        match self {
            Self::Topology => 0,
            Self::Geometry => 1,
            Self::SelfIntersection => 2,
            Self::Tolerance => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrderedPrismStageStatus {
    Blocked,
    Passed,
    Failed,
    Incomplete,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OrderedPrismStageTrace {
    states: [OrderedPrismStageStatus; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PrismStageTransitionError {
    pub actual_stage: OrderedPrismStage,
    pub actual_status: OrderedPrismStageStatus,
    pub expected_stage: Option<OrderedPrismStage>,
}

impl OrderedPrismStageTrace {
    pub(super) fn pending() -> Self {
        Self {
            states: [OrderedPrismStageStatus::Blocked; 4],
        }
    }

    /// Inspect one stage's explicit coverage; example: `trace.status(OrderedPrismStage::Geometry)`.
    pub(crate) fn status(&self, stage: OrderedPrismStage) -> OrderedPrismStageStatus {
        let index = stage.index();
        self.states[index]
    }

    pub(super) fn next(&self) -> Option<OrderedPrismStage> {
        let index = self
            .states
            .iter()
            .take_while(|s| **s == OrderedPrismStageStatus::Passed)
            .count();
        if self.states.get(index) != Some(&OrderedPrismStageStatus::Blocked) {
            return None;
        }
        Some(OrderedPrismStage::ALL[index])
    }

    pub(super) fn record_prism_stage(
        &mut self,
        stage: OrderedPrismStage,
        status: OrderedPrismStageStatus,
    ) -> Result<(), PrismStageTransitionError> {
        let expected_stage = self.next();
        if expected_stage != Some(stage) || status == OrderedPrismStageStatus::Blocked {
            return Err(PrismStageTransitionError {
                actual_stage: stage,
                actual_status: status,
                expected_stage,
            });
        }
        self.states[stage.index()] = status;
        Ok(())
    }
}
