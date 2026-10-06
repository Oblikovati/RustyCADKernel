//! Native AccAllocator bytes match whenever its four-byte cursor satisfies the typed alignment.
#![allow(unsafe_code)]
mod acc_reservation_fixture;
use acc_reservation_fixture::ReservationTrace;
use rustycad_tkernel::ncollection::BaseAllocator;

#[test]
fn aligned_native_node_sizes_consume_exact_reservations() {
    let mut trace = ReservationTrace::new("mixed", 1024);
    for size in [32, 40, 32, 40] {
        trace.request(size, 8);
    }
    trace.assert_native();
}
#[test]
fn full_blocks_create_native_sized_fresh_reservations() {
    let mut trace = ReservationTrace::new("exhaust", 96);
    for _ in 0..4 {
        trace.request(32, 8);
    }
    trace.assert_native();
}
#[test]
fn previous_block_search_preserves_native_allocation_order() {
    let mut trace = ReservationTrace::new("previous", 128);
    for size in [80, 80, 40, 32] {
        trace.request(size, 8);
    }
    trace.assert_native();
}
#[test]
fn oversized_nodes_keep_native_dedicated_block_and_reuse() {
    let mut trace = ReservationTrace::new("oversize", 128);
    for size in [160, 32] {
        trace.request(size, 8);
    }
    trace.assert_native();
}
#[test]
fn aligned_prefix_and_native_four_byte_requests_keep_cursor_bytes() {
    let mut trace = ReservationTrace::new("prefix-aligned", 96);
    for size in [8, 32, 40] {
        trace.request(size, 8);
    }
    trace.assert_native();
    let mut raw = ReservationTrace::new("unaligned-raw", 132);
    raw.request(32, 4);
    raw.request(32, 4);
    raw.assert_native();
}
#[test]
fn misaligned_geometry_adds_only_the_required_padding() {
    let arena = BaseAllocator::accumulating(132).unwrap();
    let first = arena.allocate_value([1_u64; 4]);
    assert_eq!(arena.accumulating_chain()[0].free_size, 96);
    let second = arena.allocate_value([2_u64; 4]);
    assert_eq!(arena.accumulating_chain()[0].free_size, 64);
    assert_eq!(*first, [1; 4]);
    assert_eq!(*second, [2; 4]);
    drop((first, second));
    assert_eq!(arena.accumulating_chain()[0].free_size, 132);
}
