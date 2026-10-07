//! Compare the actual protected allocator fields and reachable free lists with native traces.
use super::super::MMgrOpt;

fn assert_initialize_row(actual: &str) {
    let fixture =
        include_str!("../../../../../../tools/oracle/tkernel_mmgr_opt_initialize.expected");
    assert!(
        fixture.lines().any(|row| row == actual),
        "missing native row: {actual}"
    );
}

fn compact_pool(clear: bool) -> MMgrOpt {
    let mut allocator = MMgrOpt::new(clear, false, 17, 100, 512);
    // The native subclass uses the same legal protected-field override before allocating pools.
    allocator.page_size = 64;
    allocator
}

fn pool_topology(allocator: &MMgrOpt) -> (usize, usize, usize) {
    // Read only the actual current fields, not a parallel graph or retained snapshot cache.
    (
        allocator.alloc_list,
        allocator.next_addr,
        allocator.end_block,
    )
}

#[test]
fn initialization_preserves_empty_state_and_normalizes_constructor_parameters() {
    let mut allocator = MMgrOpt::new(false, false, 17, 7, 512);
    allocator.initialize_native_free_lists();
    assert_initialize_row(&format!(
        "normalization {} {} {} {} {}",
        allocator.cell_size,
        allocator.nb_pages,
        allocator.free_list_max,
        u8::from(allocator.page_size > 0),
        u8::from(allocator.free_lists.is_empty())
    ));
    assert_eq!(pool_topology(&allocator), (0, 0, 0));
    assert!(!allocator.clear && !allocator.mmap);
}

#[test]
fn initialization_preserves_live_pool_topology_headers_and_payload() {
    let mut allocator = compact_pool(true);
    let block = allocator.allocate(16).unwrap();
    allocator.bytes_mut(block).unwrap().fill(19);
    let before = pool_topology(&allocator);
    allocator.initialize_native_free_lists();
    assert_initialize_row(&format!(
        "live-pool {} {} {}",
        u8::from(before == pool_topology(&allocator)),
        u8::from(allocator.bytes(block).unwrap() == [19; 16]),
        u8::from(allocator.space.read_word(block.get() - 8) == Some(16))
    ));
    allocator.free(Some(block)).unwrap();
}

#[test]
fn initialization_forgets_small_free_heads_and_preserves_pool_cursor() {
    let mut allocator = compact_pool(true);
    let retired = allocator.allocate(16).unwrap();
    allocator.free(Some(retired)).unwrap();
    let before = pool_topology(&allocator);
    allocator.initialize_native_free_lists();
    let preserved = before == pool_topology(&allocator);
    let next = allocator.allocate(16).unwrap();
    assert_initialize_row(&format!(
        "forgotten-small {} {} {}",
        u8::from(preserved),
        u8::from(next != retired),
        next.get() - retired.get()
    ));
    allocator.free(Some(next)).unwrap();
    assert_initialize_row(&format!(
        "new-small-free-list {}",
        u8::from(allocator.allocate(16).unwrap() == next)
    ));
}

#[test]
fn initialization_preserves_live_medium_and_discards_recycled_medium_heads() {
    let mut allocator = compact_pool(true);
    let live = allocator.allocate(40).unwrap();
    allocator.bytes_mut(live).unwrap().fill(7);
    let retired = allocator.allocate(48).unwrap();
    allocator.free(Some(retired)).unwrap();
    allocator.initialize_native_free_lists();
    let purged = allocator.purge(false);
    let next = allocator.allocate(48).unwrap();
    assert_initialize_row(&format!(
        "forgotten-medium {} {} {}",
        purged,
        u8::from(next != retired),
        u8::from(allocator.bytes(live).unwrap() == [7; 40])
    ));
    allocator.free(Some(next)).unwrap();
    assert_initialize_row(&format!("new-medium-free-list {}", allocator.purge(false)));
}

fn assert_clearing_policy(clear: bool) {
    let mut allocator = compact_pool(clear);
    allocator.initialize_native_free_lists();
    let block = allocator.allocate(40).unwrap();
    allocator.bytes_mut(block).unwrap().fill(23);
    allocator.free(Some(block)).unwrap();
    let next = allocator.allocate(40).unwrap();
    let expected = if clear { 0 } else { 23 };
    assert_initialize_row(&format!(
        "clear-{} {} {}",
        u8::from(clear),
        u8::from(next == block),
        u8::from(allocator.bytes(next).unwrap() == [expected; 40])
    ));
}

#[test]
fn initialization_retains_native_clear_policy_for_later_reuse() {
    assert_clearing_policy(true);
    assert_clearing_policy(false);
}
