// SPDX-License-Identifier: GPL-2.0-only
//! Final callbacks precede current embedding, current frozen root and complete owned replay checks.
use super::*;

pub(super) struct AnnularJoinInputs<'a, 'r, 's> {
    pub request: &'a TwoLoopRegionRequest<'r, 's>,
    pub receipt: &'a PrismReceipt,
    pub authored: &'a PrismPublicationBindings,
}
pub(super) struct AnnularJoinWork<'a> {
    phase: AnnularFrozenPhase,
    progress: &'a mut dyn AnnularFrozenProgress,
}
/// Join the same actual annular proof to named immutable capture; `join_annular_frozen_domain(request,receipt,names,proof,options,progress)`.
/// Success certifies this correspondence; it never grants ordered body validity or complete history.
pub(in crate::sketch_region_prism) fn join_annular_frozen_domain<'r>(
    request: &TwoLoopRegionRequest<'r, '_>,
    receipt: &PrismReceipt,
    authored: &PrismPublicationBindings,
    embedding: AnnularEmbeddingEvidence,
    requested: AnnularFrozenOptions,
    progress: &mut dyn AnnularFrozenProgress,
) -> Result<AnnularFrozenDomain, Box<AnnularFrozenFailure<'r>>> {
    let inputs = AnnularJoinInputs {
        request,
        receipt,
        authored,
    };
    let mut work = AnnularJoinWork {
        phase: AnnularFrozenPhase::Admission,
        progress,
    };
    inputs
        .construct(embedding, requested, &mut work)
        .map_err(|cause| inputs.failure(work.phase, requested, cause))
}
impl<'r> AnnularJoinInputs<'_, 'r, '_> {
    fn construct(
        &self,
        embedding: AnnularEmbeddingEvidence,
        requested: AnnularFrozenOptions,
        work: &mut AnnularJoinWork<'_>,
    ) -> AnnularFrozenCheck<'r, AnnularFrozenDomain> {
        let census = census::inspect(self.authored, requested)?;
        work.poll(AnnularFrozenPhase::CurrentBinding)?;
        self.current_names(&embedding)?;
        work.phase = AnnularFrozenPhase::Capture;
        let projection = capture::project(self.receipt, self.authored)?;
        work.phase = AnnularFrozenPhase::Link;
        let operands = self.operands(&embedding, &projection, census)?;
        let joined = AnnularFrozenDomain {
            projection,
            embedding,
            census,
            faces: operands.faces,
            parents: operands.parents,
            raw_vertices: operands.raw_vertices,
            replay: operands.replay,
        };
        work.poll(AnnularFrozenPhase::Commit)?;
        self.rebind(&joined)?;
        Ok(joined)
    }
    pub(super) fn current_names(
        &self,
        embedding: &AnnularEmbeddingEvidence,
    ) -> AnnularFrozenCheck<'r, ()> {
        rebind_bound_annular_embedding(self.request, self.receipt, embedding)
            .map_err(AnnularFrozenCause::Embedding)?;
        names::inspect(self.request, embedding.binding(), self.authored)
    }
    pub(super) fn failure(
        &self,
        phase: AnnularFrozenPhase,
        requested: AnnularFrozenOptions,
        cause: AnnularFrozenCause<'r>,
    ) -> Box<AnnularFrozenFailure<'r>> {
        Box::new(AnnularFrozenFailure {
            operation: self.authored.operation,
            phase,
            requested,
            cause,
        })
    }
}
impl AnnularJoinWork<'_> {
    fn poll(&mut self, phase: AnnularFrozenPhase) -> AnnularFrozenCheck<'static, ()> {
        self.phase = phase;
        if self
            .progress
            .checkpoint(phase)
            .map_err(AnnularFrozenCause::Callback)?
        {
            return Err(AnnularFrozenCause::Cancelled { freeze_phase: None });
        }
        Ok(())
    }
}
