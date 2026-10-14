// SPDX-License-Identifier: GPL-2.0-only
//! Full non-address operand replay is assembled after actual identity linkage.
use super::*;

pub(in crate::sketch_region_prism) fn text(value: &str, bytes: &mut Vec<u8>) {
    bytes.extend((value.len() as u64).to_le_bytes());
    bytes.extend(value.as_bytes());
}
pub(in crate::sketch_region_prism) fn block(value: &[u8], bytes: &mut Vec<u8>) {
    bytes.extend((value.len() as u64).to_le_bytes());
    bytes.extend(value);
}
pub(super) fn joined_bytes(
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
    links: &JoinedProofLinks,
    selected_first: bool,
    closure: &JoinedClosureOperands,
) -> CaptureResult<Vec<u8>> {
    let mut bytes = Vec::new();
    text("martensite.same-body-prism-proof/v1", &mut bytes);
    block(&projection.replay_bytes(), &mut bytes);
    replay_options::embedding(proof.requested(), &mut bytes);
    text(
        &format!(
            "{:?}/{:?}/{:?}",
            links.contact,
            links.original,
            proof.requested()
        ),
        &mut bytes,
    );
    text(
        &format!(
            "J={};winding={};reversed={}",
            proof.binding().signed_jacobian,
            proof.binding().region.selected_winding,
            proof.binding().region.boundary_reversed
        ),
        &mut bytes,
    );
    text(
        &format!("{:?}/{:?}", links.faces, links.parents),
        &mut bytes,
    );
    raw_vertices::append(&links.raw_vertices, &mut bytes);
    replay_closure::append(closure, bound.bindings, &mut bytes)?;
    super::replay_sources::sources(proof, links, bound, &mut bytes)?;
    super::replay_sources::closure(
        &proof.binding().region,
        bound.bindings,
        selected_first,
        &mut bytes,
    )?;
    super::replay_theorem::regularity(proof, links, &mut bytes);
    super::replay_theorem::incidences(proof, bound, &mut bytes)?;
    parents::append_remaining_history(bound, &mut bytes)?;
    Ok(bytes)
}
