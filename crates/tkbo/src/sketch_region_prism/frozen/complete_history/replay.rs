// SPDX-License-Identifier: GPL-2.0-only
//! Stable authored keys order records; no lookup address, graph index or returned ordinal is serialized.
use super::*;
pub(super) fn finish(
    request: &PrismHistoryRequest<'_, '_>,
    captured: capture::CapturedPrismHistory,
) -> HistoryCheck<PrismCompleteHistory> {
    let originals = captured
        .continued
        .iter()
        .map(|r| r.original.clone())
        .collect();
    let binding = PrismHistorySourceBinding {
        operation: request.publication.operation,
        selected_region: request.publication.selected_region,
        complementary_region: request.historical.complementary_region,
        closure_contact: request.joined.closure_contact(),
        originals,
    };
    let mut result = PrismCompleteHistory {
        binding,
        requested: request.requested,
        measured_operand_rows: budget::admit(request)?,
        captured,
        joined: request.joined.replay_bytes().to_vec(),
        replay: Vec::new(),
        deleted: Vec::new(),
    };
    result.replay = encode(&result);
    Ok(result)
}
fn block(bytes: &[u8], result: &mut Vec<u8>) {
    result.extend((bytes.len() as u64).to_le_bytes());
    result.extend(bytes);
}
fn encode(value: &PrismCompleteHistory) -> Vec<u8> {
    let mut result = format!("martensite-complete-prism-history-v1;binding={:?};requested={:?};effective={:?};measured-operand-rows={:?};intermediate={:?};sweep={:?};continued={:?};original-deleted={:?};member-coverage=complete;directed-coverage=complete;whole-body-validation=not-claimed;",
        value.binding, value.requested, value.requested, value.measured_operand_rows, value.captured.intermediate, value.captured.sweep, value.captured.continued, value.deleted).into_bytes();
    block(
        &partition_metadata::replay(&value.captured.partition),
        &mut result,
    );
    block(&value.captured.lineage.replay_bytes(), &mut result);
    block(&value.joined, &mut result);
    result
}
