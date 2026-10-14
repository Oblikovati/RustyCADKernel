// SPDX-License-Identifier: GPL-2.0-only
//! Send progress adapts to unchanged V1 drivers; only owned operands are read after callbacks.
use super::*;
pub(super) struct GeometryProgress<'a, P>(pub &'a mut P);
impl<P: AnnularConformanceProgress> JoinedConformanceProgress for GeometryProgress<'_, P> {
    fn cancelled(&mut self, phase: JoinedConformancePhase) -> bool {
        self.0.cancelled(phase)
    }
}
pub(super) struct MetadataProgress<'a, P>(pub &'a mut P);
impl<P: AnnularToleranceProgress> JoinedToleranceProgress for MetadataProgress<'_, P> {
    fn cancelled(&mut self, phase: JoinedTolerancePhase) -> bool {
        self.0.cancelled(phase)
    }
}
pub(super) struct AnnularExactWork {
    pub body: ConformanceWork,
    pub source_context: Option<AnnularSourceContext>,
    pub upstream: AnnularConformanceEvidence,
}
impl AnnularExactWork {
    pub fn new() -> Self {
        Self {
            body: ConformanceWork::new(),
            source_context: None,
            upstream: AnnularConformanceEvidence::default(),
        }
    }
    pub fn evidence(mut self) -> AnnularConformanceEvidence {
        self.upstream.body = self.body.evidence;
        self.upstream
    }
    pub fn failure(
        self,
        join: &AnnularFrozenDomain,
        cause: AnnularConformanceCause,
    ) -> AnnularConformanceFailure {
        AnnularConformanceFailure {
            operation: "annular exact represented-geometry conformance V2",
            phase: self.body.phase,
            entity: self.body.entity,
            occurrence: self.body.occurrence.clone(),
            source_context: self.source_context,
            cause,
            partial: self.evidence(),
            binding: join.replay_bytes().to_vec(),
            limits: *join.projection().limits(),
        }
    }
}
