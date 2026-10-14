// SPDX-License-Identifier: GPL-2.0-only
//! ADR0051 exact versioned proof replay; full directed context references replace owner addresses.
mod domains;
mod incidences;
mod regularity;
use super::*;
use crate::sketch_region_prism::frozen::proof_join::{encode_joined_block, encode_joined_text};
use crate::sketch_region_source_conformance::encoding::numbers;

fn count(value: usize, bytes: &mut Vec<u8>) {
    bytes.extend((value as u64).to_le_bytes());
}

type ReplayCheck<T = ()> = Result<T, Box<AnnularBindingCause<'static>>>;
use crate::sketch_region::RegionObservedContext;
fn context(value: &RegionObservedContext, bytes: &mut Vec<u8>) -> ReplayCheck {
    encode_annular_context(value, bytes)
}
pub(super) fn encode(value: &AnnularEmbeddingEvidence) -> ReplayCheck<Vec<u8>> {
    let mut bytes = Vec::new();
    encode_joined_text("martensite.annular-prism-embedding/v2", &mut bytes);
    encode_joined_block(value.binding.replay_bytes(), &mut bytes);
    bytes.extend(value.requested.version.to_le_bytes());
    regularity::policy(value.requested.regularity, &mut bytes);
    regularity::encode(&value.regularity, &mut bytes);
    let domains = &value.domains;
    for face in [
        &domains.first,
        &domains.last,
        &domains.outer,
        &domains.inner,
    ] {
        domains::domain(face, &value.binding, &mut bytes)?;
    }
    for row in &value.incidences {
        incidences::encode(row, &value.binding, &mut bytes)?;
    }
    Ok(bytes)
}
