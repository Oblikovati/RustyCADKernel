// SPDX-License-Identifier: GPL-2.0-only
use super::{work::OrderedWork, *};
use crate::sketch_region_prism::exact_tolerance::*;
use rustycad_tkernel::standard::Failure;

impl topology::TopologyProgress for OrderedWork<'_> {
    fn checkpoint(&mut self, point: topology::TopologyCheckpoint) -> Result<bool, Failure> {
        let cancelled = self.poll(OrderedPrismCheck::Topology(point));
        Ok(cancelled)
    }
}
impl JoinedConformanceProgress for OrderedWork<'_> {
    fn cancelled(&mut self, phase: JoinedConformancePhase) -> bool {
        let check = OrderedPrismCheck::Conformance(phase);
        self.poll(check)
    }
}
impl JoinedToleranceProgress for OrderedWork<'_> {
    fn cancelled(&mut self, phase: JoinedTolerancePhase) -> bool {
        let check = OrderedPrismCheck::Tolerance(phase);
        self.poll(check)
    }
}
