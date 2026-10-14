// SPDX-License-Identifier: GPL-2.0-only
//! One owned join, followed by a callback-free replay of every current operand.
use super::*;
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_prism::PrismReceipt;

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
    let wrap = |phase, cause| failure(bindings.operation, phase, cause);
    admit(requested).map_err(|cause| wrap(PrismProofJoinPhase::Admission, cause))?;
    let initial = probe::capture(bridge, sources, receipt, bindings)
        .map_err(|cause| wrap(PrismProofJoinPhase::Admission, cause))?;
    let embedding = inspect_prism_embedding(
        bridge,
        sources,
        receipt,
        requested.embedding,
        &mut cancelled,
    )
    .map_err(|cause| {
        wrap(
            PrismProofJoinPhase::Embedding,
            PrismProofJoinCause::Embedding(Box::new(cause)),
        )
    })?;
    let projection = capture_projection(receipt, bindings, &mut cancelled)
        .map_err(|cause| wrap(PrismProofJoinPhase::Capture, cause))?;
    let join = assemble(
        bridge,
        sources,
        receipt,
        bindings,
        contact,
        requested,
        (embedding, projection),
    )
    .map_err(|cause| wrap(PrismProofJoinPhase::Link, cause))?;
    final_poll(&mut cancelled).map_err(|cause| wrap(PrismProofJoinPhase::Commit, cause))?;
    rebind_joined_prism_proof(bridge, sources, receipt, bindings, &join)?;
    let current = probe::capture(bridge, sources, receipt, bindings)
        .map_err(|cause| wrap(PrismProofJoinPhase::Commit, cause))?;
    probe::same(&initial, &current).map_err(|cause| wrap(PrismProofJoinPhase::Commit, cause))?;
    Ok(join)
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
    match super::super::freeze_prism_with_behavior(
        receipt,
        bindings,
        &mut JoinFreezeProgress(cancelled),
        PrismFreezeBehavior::OrderedTraversalV2,
    ) {
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
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
    contact: &AuthoredClosureContact,
    requested: PrismProofJoinOptions,
    proof: (PrismEmbeddingEvidence, FrozenPrismProjection),
) -> Result<JoinedPrismProof, PrismProofJoinCause> {
    let (embedding, projection) = proof;
    let bound = bind_projection(receipt, bindings).map_err(link)?;
    let links = links::link_proof(&embedding, &bound, &projection, contact).map_err(link)?;
    let closure = closure::capture(bridge, sources, receipt, &embedding, bindings, contact)?;
    let first =
        replay_sources::first_is_selected(bridge, &embedding.binding().region).map_err(link)?;
    let replay = replay::joined_bytes(&embedding, &bound, &projection, &links, first, &closure)
        .map_err(link)?;
    Ok(JoinedPrismProof {
        projection,
        embedding,
        links,
        requested,
        replay,
        closure,
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
    rebind(bridge, sources, receipt, bindings, join)
        .map_err(|cause| failure(bindings.operation, PrismProofJoinPhase::Commit, cause))
}
fn rebind(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
    join: &JoinedPrismProof,
) -> Result<(), PrismProofJoinCause> {
    rebind_current_prism_binding(bridge, sources, receipt, join.embedding.binding())
        .map_err(|cause| PrismProofJoinCause::Binding(Box::new(cause)))?;
    let bound = bind_projection(receipt, bindings).map_err(link)?;
    super::super::capture::check_final_capture(
        &bound,
        &join.projection,
        PrismFreezeBehavior::OrderedTraversalV2,
    )
    .map_err(link)?;
    let contact = AuthoredClosureContact {
        name: join.links.contact,
        actual: join.embedding.binding().region.contact,
        original: join.embedding.binding().region.original.clone(),
    };
    let links =
        links::link_proof(&join.embedding, &bound, &join.projection, &contact).map_err(link)?;
    let closure = closure::capture(
        bridge,
        sources,
        receipt,
        &join.embedding,
        bindings,
        &contact,
    )?;
    let first = replay_sources::first_is_selected(bridge, &join.embedding.binding().region)
        .map_err(link)?;
    let replay = replay::joined_bytes(
        &join.embedding,
        &bound,
        &join.projection,
        &links,
        first,
        &closure,
    )
    .map_err(link)?;
    if replay == join.replay {
        return Ok(());
    }
    Err(PrismProofJoinCause::Stale {
        representation: "complete authored join and raw/query operands",
    })
}
fn link(cause: PrismFreezeCause) -> PrismProofJoinCause {
    PrismProofJoinCause::Link(Box::new(cause))
}
