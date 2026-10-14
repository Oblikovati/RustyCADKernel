// SPDX-License-Identifier: GPL-2.0-only
//! Complete bytes bind exact caller dictionary, legacy child and ordered semantic correspondence.
use super::super::{body_keys::*, replay::word, *};
use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenSweepRole, FrozenUseKey};
pub(super) fn encode(
    envelope: &SemanticReplayEnvelope,
    members: &[SemanticMemberBinding],
    uses: &[SemanticUseBinding],
) -> Vec<u8> {
    let mut bytes = b"Martensite.SemanticBodyMap\0".to_vec();
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    word(&mut bytes, envelope.replay_bytes().len() as u64);
    bytes.extend_from_slice(envelope.replay_bytes());
    word(&mut bytes, members.len() as u64);
    for row in members {
        member_row(&mut bytes, row);
    }
    word(&mut bytes, uses.len() as u64);
    for row in uses {
        use_row(&mut bytes, row);
    }
    bytes
}
fn member_row(bytes: &mut Vec<u8>, row: &SemanticMemberBinding) {
    encode_frozen_member_key(bytes, row.frozen);
    encode_semantic_member_key(bytes, &row.semantic);
    bytes.push(u8::from(row.output_slot.is_some()));
    if let Some(value) = row.output_slot {
        slot(bytes, value);
    }
}
fn use_row(bytes: &mut Vec<u8>, row: &SemanticUseBinding) {
    frozen_use(bytes, &row.frozen);
    encode_semantic_use_key(bytes, &row.semantic);
}
/// Encode the complete ancestry without lookup; `encode_semantic_use_key(bytes, key)`.
pub(crate) fn encode_semantic_use_key(bytes: &mut Vec<u8>, key: &SemanticUseKey) {
    word(bytes, key.0.len() as u64);
    for step in &key.0 {
        encode_semantic_member_key(bytes, &step.member);
        slot(bytes, step.directed_use);
    }
}
/// Encode the complete authored member observation; `encode_frozen_member_key(bytes, key)` requires separate caller-context binding.
pub(crate) fn encode_frozen_member_key(bytes: &mut Vec<u8>, key: FrozenPrismKey) {
    word(bytes, key.operation.0);
    word(bytes, key.source.0);
    bytes.extend_from_slice(&[role(key.role), key.kind]);
}
fn frozen_use(bytes: &mut Vec<u8>, key: &FrozenUseKey) {
    word(bytes, key.0.len() as u64);
    for (member, token) in &key.0 {
        encode_frozen_member_key(bytes, *member);
        word(bytes, token.0);
    }
}
/// Encode caller-qualified source semantics; `encode_semantic_member_key(bytes, key)`.
pub(crate) fn encode_semantic_member_key(bytes: &mut Vec<u8>, key: &SemanticMemberKey) {
    word(bytes, key.operation.0);
    word(bytes, key.source.0);
    slot(bytes, key.source_member);
    bytes.extend_from_slice(&[role(key.sweep_role), key.kind]);
}
fn slot(bytes: &mut Vec<u8>, value: SemanticNameSlot) {
    bytes.push(value.stage as u8);
    bytes.push(value.role as u8);
}
fn role(value: FrozenSweepRole) -> u8 {
    match value {
        FrozenSweepRole::First => 1,
        FrozenSweepRole::Last => 2,
        FrozenSweepRole::Generated => 3,
    }
}
