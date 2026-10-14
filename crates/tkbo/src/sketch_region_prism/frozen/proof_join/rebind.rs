// SPDX-License-Identifier: GPL-2.0-only
//! Preserve the callback-free sequence: current proof, frozen projection, then complete join.
use super::operation::JoinInputs;
use super::*;

pub(super) fn check(
    inputs: &JoinInputs<'_, '_>,
    join: &JoinedPrismProof,
) -> Result<(), PrismProofJoinCause> {
    let bound = current_projection(inputs, join)?;
    let replay = current_join(inputs, &bound, join)?;
    if replay == join.replay {
        return Ok(());
    }
    Err(PrismProofJoinCause::Stale {
        representation: "complete authored join and raw/query operands",
    })
}
fn current_projection<'a>(
    inputs: &JoinInputs<'a, '_>,
    join: &JoinedPrismProof,
) -> Result<BoundProjection<'a>, PrismProofJoinCause> {
    rebind_current_prism_binding(
        inputs.bridge,
        inputs.sources,
        inputs.receipt,
        join.embedding.binding(),
    )
    .map_err(|cause| PrismProofJoinCause::Binding(Box::new(cause)))?;
    let bound = bind_projection(inputs.receipt, inputs.bindings).map_err(join_link_cause)?;
    super::super::capture::check_final_capture(
        &bound,
        &join.projection,
        PrismFreezeBehavior::OrderedTraversalV2,
    )
    .map_err(join_link_cause)?;
    Ok(bound)
}
fn current_join(
    inputs: &JoinInputs<'_, '_>,
    bound: &BoundProjection<'_>,
    join: &JoinedPrismProof,
) -> Result<Vec<u8>, PrismProofJoinCause> {
    let contact = retained_contact(join);
    operands::capture(inputs, bound, &join.embedding, &join.projection, &contact)
        .map(|captured| captured.replay)
}

fn retained_contact(join: &JoinedPrismProof) -> AuthoredClosureContact {
    AuthoredClosureContact {
        name: join.links.contact,
        actual: join.embedding.binding().region.contact,
        original: join.embedding.binding().region.original.clone(),
    }
}
