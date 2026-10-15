// SPDX-License-Identifier: GPL-2.0-only
//! Length-delimited full child receipts retain every policy and authored identity.
use super::PublishedAnnularPrism;

pub(super) fn encode(result: &PublishedAnnularPrism) -> Vec<u8> {
    let mut bytes = b"Martensite complete named annular prism publication V1\n".to_vec();
    let requested = format!("{:?}", result.requested);
    append(&mut bytes, requested.as_bytes());
    append(&mut bytes, result.join.replay_bytes());
    append(&mut bytes, &result.validation.replay_bytes());
    append(&mut bytes, result.history.replay_bytes());
    append(&mut bytes, result.measures.replay_bytes());
    bytes
}
fn append(bytes: &mut Vec<u8>, receipt: &[u8]) {
    bytes.extend_from_slice(&(receipt.len() as u64).to_le_bytes());
    bytes.extend_from_slice(receipt);
}
