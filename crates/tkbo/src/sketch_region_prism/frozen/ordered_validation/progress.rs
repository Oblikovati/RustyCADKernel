// SPDX-License-Identifier: GPL-2.0-only
use super::{OrderedPrismStage, topology::*};
use crate::sketch_region_prism::exact_tolerance::*;
use rustycad_tkernel::standard::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrderedPrismCheck {
    Admission,
    Topology(TopologyCheckpoint),
    Orientation { commit: bool },
    Conformance(JoinedConformancePhase),
    Incidence { completed: usize },
    Tolerance(JoinedTolerancePhase),
    Commit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OrderedPrismCheckpoint {
    pub stage: OrderedPrismStage,
    pub check: OrderedPrismCheck,
    pub ordinal: usize,
}
pub(crate) trait OrderedPrismValidationProgress: Send {
    /// Refuse or cancel before accepting more evidence; example: `progress.checkpoint(point)`.
    fn checkpoint(&mut self, point: OrderedPrismCheckpoint) -> Result<bool, Failure>;
}
