// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkernel::standard::Failure;
impl topology::TopologyProgress for AnnularValidationContext<'_> {
    fn checkpoint(&mut self, point: topology::TopologyCheckpoint) -> Result<bool, Failure> {
        Ok(self.poll(AnnularValidationCheck::Topology(point)))
    }
}
impl AnnularConformanceProgress for AnnularValidationContext<'_> {
    fn cancelled(&mut self, phase: JoinedConformancePhase) -> bool {
        self.poll(AnnularValidationCheck::Conformance(phase))
    }
}
impl AnnularToleranceProgress for AnnularValidationContext<'_> {
    fn cancelled(&mut self, phase: JoinedTolerancePhase) -> bool {
        self.poll(AnnularValidationCheck::Tolerance(phase))
    }
}
