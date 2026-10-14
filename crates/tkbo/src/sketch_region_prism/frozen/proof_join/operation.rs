// SPDX-License-Identifier: GPL-2.0-only
//! One owned join, followed by a callback-free replay of every current operand.
use super::*;
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_prism::PrismReceipt;

pub(super) struct JoinInputs<'a, 'b> {
    pub bridge: &'a PairedContactTransaction,
    pub sources: &'a [ArrangementSource<'b>],
    pub receipt: &'a PrismReceipt,
    pub bindings: &'a PrismPublicationBindings,
}
/// Compose current proof and immutable ordered projection; `compose_prism_proof(bridge,sources,receipt,bindings,contact,options,cancel)`.
/// Exclusive operation-local inputs are required; success is evidence, never accepted body status.
pub(crate) fn compose_prism_proof(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
    contact: &AuthoredClosureContact,
    requested: PrismProofJoinOptions,
    mut cancelled: impl FnMut() -> bool + Send,
) -> Result<JoinedPrismProof, PrismProofJoinFailure> {
    let inputs = JoinInputs {
        bridge,
        sources,
        receipt,
        bindings,
    };
    inputs.compose(contact, requested, &mut cancelled)
}
impl JoinInputs<'_, '_> {
    fn compose(
        &self,
        contact: &AuthoredClosureContact,
        requested: PrismProofJoinOptions,
        cancelled: &mut (impl FnMut() -> bool + Send),
    ) -> Result<JoinedPrismProof, PrismProofJoinFailure> {
        admit(requested).map_err(|cause| self.failure(PrismProofJoinPhase::Admission, cause))?;
        let initial = self.raw_probe(PrismProofJoinPhase::Admission)?;
        let embedding = self.embedding(requested, cancelled)?;
        let projection = capture_projection(self.receipt, self.bindings, cancelled)
            .map_err(|cause| self.failure(PrismProofJoinPhase::Capture, cause))?;
        let join = assemble(self, contact, requested, embedding, projection)
            .map_err(|cause| self.failure(PrismProofJoinPhase::Link, cause))?;
        self.finish(&join, &initial, cancelled)?;
        Ok(join)
    }
    fn embedding(
        &self,
        requested: PrismProofJoinOptions,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<PrismEmbeddingEvidence, PrismProofJoinFailure> {
        inspect_prism_embedding(
            self.bridge,
            self.sources,
            self.receipt,
            requested.embedding,
            cancelled,
        )
        .map_err(|cause| {
            self.failure(
                PrismProofJoinPhase::Embedding,
                PrismProofJoinCause::Embedding(Box::new(cause)),
            )
        })
    }
    fn finish(
        &self,
        join: &JoinedPrismProof,
        initial: &probe::InitialRawProbe,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), PrismProofJoinFailure> {
        final_poll(cancelled).map_err(|cause| self.failure(PrismProofJoinPhase::Commit, cause))?;
        rebind_joined_prism_proof(self.bridge, self.sources, self.receipt, self.bindings, join)?;
        let current = self.raw_probe(PrismProofJoinPhase::Commit)?;
        probe::same(initial, &current)
            .map_err(|cause| self.failure(PrismProofJoinPhase::Commit, cause))
    }
    fn raw_probe(
        &self,
        phase: PrismProofJoinPhase,
    ) -> Result<probe::InitialRawProbe, PrismProofJoinFailure> {
        probe::capture(self.bridge, self.sources, self.receipt, self.bindings)
            .map_err(|cause| self.failure(phase, cause))
    }
    pub(super) fn failure(
        &self,
        phase: PrismProofJoinPhase,
        cause: PrismProofJoinCause,
    ) -> PrismProofJoinFailure {
        failure(self.bindings.operation, phase, cause)
    }
}
fn admit(requested: PrismProofJoinOptions) -> Result<(), PrismProofJoinCause> {
    if requested.version == 1 {
        return Ok(());
    }
    Err(PrismProofJoinCause::Version {
        actual: requested.version,
        expected: 1,
    })
}
fn final_poll(cancelled: &mut impl FnMut() -> bool) -> Result<(), PrismProofJoinCause> {
    if !cancelled() {
        return Ok(());
    }
    Err(PrismProofJoinCause::Cancelled { freeze_phase: None })
}
struct JoinFreezeProgress<'a, F>(&'a mut F);
impl<F: FnMut() -> bool + Send> FreezeProgress for JoinFreezeProgress<'_, F> {
    fn checkpoint(&mut self, _: FreezePhase) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok((self.0)())
    }
}
fn capture_projection(
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
    cancelled: &mut (impl FnMut() -> bool + Send),
) -> Result<FrozenPrismProjection, PrismProofJoinCause> {
    let captured = super::super::freeze_prism_with_behavior(
        receipt,
        bindings,
        &mut JoinFreezeProgress(cancelled),
        PrismFreezeBehavior::OrderedTraversalV2,
    );
    capture_outcome(captured)
}
fn capture_outcome(
    captured: PrismFreezeOutcome,
) -> Result<FrozenPrismProjection, PrismProofJoinCause> {
    match captured {
        PrismFreezeOutcome::Captured(projection) => Ok(*projection),
        PrismFreezeOutcome::Incomplete(cause) => {
            Err(PrismProofJoinCause::IncompleteCapture(Box::new(cause)))
        }
        PrismFreezeOutcome::Failed(cause) => {
            Err(PrismProofJoinCause::FailedCapture(Box::new(cause)))
        }
        PrismFreezeOutcome::Cancelled { phase, .. } => Err(PrismProofJoinCause::Cancelled {
            freeze_phase: Some(phase),
        }),
    }
}
fn assemble(
    inputs: &JoinInputs<'_, '_>,
    contact: &AuthoredClosureContact,
    requested: PrismProofJoinOptions,
    embedding: PrismEmbeddingEvidence,
    projection: FrozenPrismProjection,
) -> Result<JoinedPrismProof, PrismProofJoinCause> {
    let bound = bind_projection(inputs.receipt, inputs.bindings).map_err(join_link_cause)?;
    let captured = operands::capture(inputs, &bound, &embedding, &projection, contact)?;
    Ok(JoinedPrismProof {
        projection,
        embedding,
        links: captured.links,
        requested,
        replay: captured.replay,
        closure: captured.closure,
    })
}
/// Recheck the same owned join after downstream callbacks; `rebind_joined_prism_proof(bridge,sources,receipt,bindings,join)`.
/// This performs no external callback and grants no additional validation coverage.
pub(crate) fn rebind_joined_prism_proof(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
    join: &JoinedPrismProof,
) -> Result<(), PrismProofJoinFailure> {
    let inputs = JoinInputs {
        bridge,
        sources,
        receipt,
        bindings,
    };
    rebind::check(&inputs, join).map_err(|cause| inputs.failure(PrismProofJoinPhase::Commit, cause))
}
