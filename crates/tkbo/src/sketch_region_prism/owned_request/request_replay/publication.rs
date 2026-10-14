// SPDX-License-Identifier: GPL-2.0-only
//! Reuse existing child option encoders before any child certificate exists.
use super::*;
use crate::sketch_region_prism::{
    exact_tolerance::encode_stored_tolerance_cap,
    frozen::{
        complete_history::PrismHistoryOptions, joined_measures::*,
        proof_join::encode_joined_embedding_options,
    },
    publication::PrismPublicationOptions,
};
/// Compare exact invocation policy bits; `encode_publication_request_options(options)` includes refused inputs.
pub(crate) fn encode_publication_request_options(options: PrismPublicationOptions) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(options.version.to_le_bytes());
    bytes.extend(options.join.version.to_le_bytes());
    encode_joined_embedding_options(options.join.embedding, &mut bytes);
    bytes.extend(options.validation.version.to_le_bytes());
    encode_stored_tolerance_cap(options.validation.stored_cap, &mut bytes);
    history(options.history, &mut bytes);
    measures(options.measures, &mut bytes);
    bytes
}
fn history(options: PrismHistoryOptions, bytes: &mut Vec<u8>) {
    bytes.extend(options.version.to_le_bytes());
    word(options.limits.maximum_depth as u64, bytes);
    word(options.limits.maximum_uses as u64, bytes);
    word(options.limits.maximum_rows as u64, bytes);
}
fn measures(options: JoinedPrismMeasureOptions, bytes: &mut Vec<u8>) {
    bytes.extend(options.version.to_le_bytes());
    encode_joined_integral_options(options.scalars.subset.cap_area, bytes);
    encode_joined_integral_options(options.scalars.subset.signed_volume, bytes);
    encode_joined_side_options(options.scalars.side_area, bytes);
    encode_joined_total_options(options.scalars.total_area, bytes);
}
