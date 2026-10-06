//! Protected operations consume actual intrusive arena headers, preserving native list phases.
use rustycad_tkernel::ncollection::{BaseAllocator, IncAllocatorObservation, IncBlockObservation};

fn assert_native_row(actual: &str) {
    let fixture =
        include_str!("../../../tools/oracle/tkernel_inc_allocator_block_operations.expected");
    assert!(
        fixture.lines().any(|expected| expected == actual),
        "missing native row: {actual}"
    );
}

fn selected_block(state: &IncAllocatorObservation, address: usize) -> IncBlockObservation {
    let block = state.blocks.iter().find(|block| block.address == address);
    *block.expect("observed identity: expected a live header")
}

#[test]
fn reset_used_block_restores_bytes_without_rejoining_allocation_heap() {
    let arena = BaseAllocator::incremental(2048);
    let mut access = arena.incremental_extension().unwrap();
    let first = access.allocate_slow(2048).unwrap();
    let used = access.observation().used_heap;
    access.reset_block(used).unwrap();
    let next = access.allocate_slow(8).unwrap();
    let after = access.observation();
    let block = selected_block(&after, used);
    assert_native_row(&format!(
        "reset-used {} {} {} {} {}",
        u8::from(block.current == first.as_ptr().addr()),
        block.available,
        u8::from(after.used_heap == used),
        u8::from(next == first),
        after.block_count
    ));
}

#[test]
fn reset_available_block_reuses_payload_without_reordering_head() {
    let arena = BaseAllocator::incremental(2048);
    let mut access = arena.incremental_extension().unwrap();
    let first = access.allocate_slow(80).unwrap();
    let head = access.observation().allocation_heap;
    access.reset_block(head).unwrap();
    let next = access.allocate_slow(16).unwrap();
    let after = access.observation();
    assert_native_row(&format!(
        "reset-available {} {} {}",
        u8::from(next == first),
        after.blocks[0].available,
        u8::from(head == after.allocation_heap)
    ));
}

fn clean_metadata_row(enabled: bool, state: &IncAllocatorObservation) -> String {
    format!(
        "clean-{} {} {} {} {} {} {}",
        u8::from(enabled),
        state.block_size,
        state.block_count,
        u8::from(state.allocation_heap == 0),
        u8::from(state.used_heap == 0),
        u8::from(state.ordered_blocks == 0),
        u8::from(state.thread_safe)
    )
}

#[test]
fn clean_releases_creation_chain_and_preserves_native_policy() {
    for enabled in [false, true] {
        let arena = BaseAllocator::incremental(4096);
        arena.set_thread_safe(enabled).unwrap();
        let mut access = arena.incremental_extension().unwrap();
        access.allocate_slow(4096).unwrap();
        access.allocate_slow(8).unwrap();
        access.clean().unwrap();
        let row = clean_metadata_row(enabled, &access.observation());
        let first = access.allocate_slow(4096).unwrap();
        let next = access.allocate_slow(8192).unwrap();
        assert_native_row(&format!(
            "{row} {}",
            u8::from(next.as_ptr().addr() == first.as_ptr().addr() + 4096)
        ));
    }
}

#[test]
fn invalid_header_identity_never_becomes_a_dereference() {
    let arena = BaseAllocator::incremental(2048);
    let mut access = arena.incremental_extension().unwrap();
    let payload = access.allocate_slow(8).unwrap();
    let before = access.observation();
    for address in [0, 1, usize::MAX, payload.as_ptr().addr()] {
        let error = access.reset_block(address).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_NoSuchObject");
        assert!(error.message().contains(&format!("{address:#x}")));
        assert_eq!(access.observation(), before);
    }
    access.clean().unwrap();
    assert!(access.reset_block(before.ordered_blocks).is_err());
}

#[test]
fn live_lease_rejects_all_destructive_protected_operations_atomically() {
    let arena = BaseAllocator::incremental(2048);
    let owned = arena.allocate_value(41_u64);
    let mut access = arena.incremental_extension().unwrap();
    let before = access.observation();
    assert!(access.reset_block(before.ordered_blocks).is_err());
    assert!(access.clean().is_err());
    assert_eq!(access.observation(), before);
    drop(access);
    assert_eq!(*owned, 41);
    drop(owned);
    arena.incremental_extension().unwrap().clean().unwrap();
}

#[path = "inc_allocator_block_operations/safety.rs"]
mod safety;
#[path = "inc_allocator_block_operations/topology.rs"]
mod topology;
