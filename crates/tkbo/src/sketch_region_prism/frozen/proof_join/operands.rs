// SPDX-License-Identifier: GPL-2.0-only
//! Initial assembly and final rebind share the same named operand capture and encoding.
use super::operation::JoinInputs;
use super::*;

pub(super) struct JoinedOperands {
    pub links: JoinedProofLinks,
    pub closure: JoinedClosureOperands,
    pub replay: Vec<u8>,
}
pub(super) fn capture(
    inputs: &JoinInputs<'_, '_>,
    bound: &BoundProjection<'_>,
    embedding: &PrismEmbeddingEvidence,
    projection: &FrozenPrismProjection,
    contact: &AuthoredClosureContact,
) -> Result<JoinedOperands, PrismProofJoinCause> {
    let links =
        links::link_proof(embedding, bound, projection, contact).map_err(join_link_cause)?;
    let closure = current_closure(inputs, embedding, contact)?;
    let first = replay_sources::first_is_selected(inputs.bridge, &embedding.binding().region)
        .map_err(join_link_cause)?;
    let replay = replay::joined_bytes(embedding, bound, projection, &links, first, &closure)
        .map_err(join_link_cause)?;
    Ok(JoinedOperands {
        links,
        closure,
        replay,
    })
}
fn current_closure(
    inputs: &JoinInputs<'_, '_>,
    embedding: &PrismEmbeddingEvidence,
    contact: &AuthoredClosureContact,
) -> Result<JoinedClosureOperands, PrismProofJoinCause> {
    closure::capture(
        inputs.bridge,
        inputs.sources,
        inputs.receipt,
        embedding,
        inputs.bindings,
        contact,
    )
}
