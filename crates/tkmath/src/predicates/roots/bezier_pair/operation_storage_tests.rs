// SPDX-License-Identifier: GPL-2.0-only
//! Current payload layout is measured separately from allocator totals and peak RSS.
use super::*;
use std::mem::{align_of, size_of};
#[test]
fn actual_box_payload_layout_is_reported_without_claiming_allocator_counts() {
    let pointer_bytes = size_of::<usize>();
    assert_eq!(size_of::<Box<GraphProofEvidence>>(), pointer_bytes);
    assert_eq!(size_of::<Box<PreparedProjection>>(), pointer_bytes);
    assert_eq!(size_of::<Box<ConstantPrepared>>(), pointer_bytes);
    assert!(size_of::<GraphProofEvidence>() > pointer_bytes);
    assert!(size_of::<PreparedProjection>() > pointer_bytes);
    println!(
        "{{\"format\":1,\"pointer_bytes\":{},\"graph_payload_bytes\":{},\"prepared_graph_payload_bytes\":{},\"constant_preparation_payload_bytes\":{},\"proof_enum_bytes\":{},\"preparation_enum_bytes\":{},\"retained_proof_bytes\":{},\"payload_alignment\":{},\"allocator_counts_measured\":false,\"comparative_time_rss_measured\":false}}",
        pointer_bytes,
        size_of::<GraphProofEvidence>(),
        size_of::<PreparedProjection>(),
        size_of::<ConstantPrepared>(),
        size_of::<PairProofEvidence>(),
        size_of::<PreparedEvidence>(),
        size_of::<PairOperationProof>(),
        align_of::<GraphProofEvidence>(),
    );
}
