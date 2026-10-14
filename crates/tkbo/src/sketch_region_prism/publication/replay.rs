// SPDX-License-Identifier: GPL-2.0-only
//! Length-delimited full child receipts retain all policy bits and authored identities.
use super::result::PublishedNamedPrism;
pub(super) fn encode(result: &PublishedNamedPrism) -> Vec<u8> {
    let mut bytes = b"Martensite complete named prism publication V1\n".to_vec();
    bytes.extend_from_slice(&result.requested.version.to_le_bytes());
    append(&mut bytes, result.join.replay_bytes());
    append(&mut bytes, &result.validation.replay_bytes());
    append(&mut bytes, result.history.replay_bytes());
    append(&mut bytes, &result.measures.replay_bytes());
    bytes
}
fn append(bytes: &mut Vec<u8>, receipt: &[u8]) {
    bytes.extend_from_slice(&(receipt.len() as u64).to_le_bytes());
    bytes.extend_from_slice(receipt);
}
