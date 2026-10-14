// SPDX-License-Identifier: GPL-2.0-only
//! The embedding owner supplies its complete proof encoding; raw lookup identities never enter keys.
use super::*;
pub(super) fn encode(
    proof: &AnnularEmbeddingEvidence,
    projection: &FrozenPrismProjection,
    census: AnnularNameCensus,
    operands: &capture::AnnularJoinedOperands,
    bound: &BoundProjection<'_>,
) -> CaptureResult<Vec<u8>> {
    let mut bytes = Vec::new();
    encode_joined_text("martensite.named-annular-frozen-domain/v2", &mut bytes);
    encode_joined_block(proof.replay_bytes(), &mut bytes);
    encode_joined_block(&projection.replay_bytes(), &mut bytes);
    encode_joined_text(&format!("{census:?}"), &mut bytes);
    encode_joined_text(
        &format!("{:?}/{:?}", operands.faces, operands.parents),
        &mut bytes,
    );
    encode_frozen_raw_vertices(&operands.raw_vertices, &mut bytes);
    append_frozen_remaining_history(bound, &mut bytes)?;
    Ok(bytes)
}
