// SPDX-License-Identifier: GPL-2.0-only
//! Corrupt only owned observations; never mutate the caller or mapped algorithm to create failures.
use super::{fixtures::*, spans::*};

#[test]
fn actual_native_spans_are_structurally_contiguous_not_a_contact_certificate() {
    let snapshot = actual_binding_snapshot(&two_root_edges());
    for source in &snapshot.sources {
        assert_eq!(
            inspect_native_spans(source),
            Ok(NativeSpanAudit::Contiguous { blocks: 3 })
        );
    }
}
#[test]
fn missing_referenced_blocks_are_not_completed_empty_partition() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].blocks.clear();
    assert_eq!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::MissingBlocks)
    );
}
#[test]
fn native_observation_gap_is_rejected_without_widening_bounds() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].blocks[1].own_range[0] = 0.25;
    assert!(matches!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::Boundary { position: 1, .. })
    ));
}
#[test]
fn native_observation_overlap_is_rejected_without_sorting_or_trimming() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].blocks[1].own_range[0] = 0.;
    assert!(matches!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::Boundary { position: 1, .. })
    ));
}
#[test]
fn unreferenced_flag_cannot_hide_observed_pave_blocks() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].has_reference = false;
    assert_eq!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::UnexpectedBlocks { count: 3 })
    );
}
#[test]
fn reversed_and_nonfinite_observation_ranges_keep_actual_causes() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].range = [1., 0.];
    assert_eq!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::Reversed { range: [1., 0.] })
    );
    snapshot.sources[0].range = [0., f64::INFINITY];
    assert_eq!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::Nonfinite {
            role: "range end",
            value: f64::INFINITY
        })
    );
}

#[test]
fn duplicated_span_observation_cannot_pass_by_native_done_or_adjacency() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    let duplicate = snapshot.sources[0].blocks[0];
    snapshot.sources[0].blocks.insert(1, duplicate);
    assert!(matches!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::Boundary { position: 1, .. })
    ));
}

#[test]
fn missing_terminal_native_span_is_not_a_partial_success() {
    let mut snapshot = actual_binding_snapshot(&two_root_edges());
    snapshot.sources[0].blocks.pop();
    assert!(matches!(
        inspect_native_spans(&snapshot.sources[0]),
        Err(NativeSpanCause::Boundary { position: 2, .. })
    ));
}
