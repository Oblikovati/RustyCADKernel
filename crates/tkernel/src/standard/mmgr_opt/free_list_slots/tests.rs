//! Native table layout and engine transition comparisons live in the following focused tests.
use super::NativeFreeListSlots;

#[test]
fn contiguous_slots_are_zeroed_checked_and_retain_prior_allocations() {
    let mut slots = NativeFreeListSlots::default();
    slots.replace_with_zeroed(64);
    let old = slots.active.as_ref().unwrap().as_slice().as_ptr().addr();
    slots.set_head(2, 123).unwrap();
    slots.replace_with_zeroed(64);
    assert_ne!(
        slots.active.as_ref().unwrap().as_slice().as_ptr().addr(),
        old
    );
    assert_eq!(slots.head(2).unwrap(), 0);
    assert_eq!(slots.retired.len(), 1);
    assert_eq!(
        super::decode_slot(&slots.retired[0].as_slice()[16..24]),
        123
    );
    assert!(slots.head(65).is_err());
}

#[test]
fn overflowing_table_extent_retains_null_state_and_old_backing_without_fallback() {
    let mut slots = NativeFreeListSlots::default();
    slots.replace_with_zeroed(1);
    slots.set_head(1, 123).unwrap();
    slots.replace_with_zeroed(usize::MAX / 8);
    assert!(slots.active.is_none());
    assert_eq!(slots.retired.len(), 1);
    assert!(slots.head(0).is_err());
    assert!(slots.heads().is_err());
    assert!(slots.set_head(0, 456).is_err());
    assert!(!slots.is_empty());
}

use super::super::{MMgrOpt, StorageAddress};

fn assert_table_row(actual: &str) {
    let fixture =
        include_str!("../../../../../../tools/oracle/tkernel_mmgr_opt_free_list_table.expected");
    assert!(
        fixture.lines().any(|row| row == actual),
        "missing native row: {actual}"
    );
}

fn assert_table_layout(threshold: usize) {
    let allocator = MMgrOpt::new(true, false, 0, 100, threshold);
    let table = allocator.free_lists.active.as_ref().unwrap();
    assert_table_row(&format!(
        "table-{threshold} {} {} {} {}",
        allocator.free_list_max,
        table.len(),
        u8::from(table.as_slice().iter().all(|byte| *byte == 0)),
        u8::from(table.as_slice().as_ptr().addr().is_multiple_of(8))
    ));
}

#[test]
fn native_thresholds_define_actual_contiguous_table_extents() {
    for threshold in [0, 1, 7, 8, 9, 511, 512, 513, 514] {
        assert_table_layout(threshold);
    }
}

fn recycled_pair(allocator: &mut MMgrOpt) -> [StorageAddress; 2] {
    let first = allocator.allocate(9).unwrap();
    let second = allocator.allocate(13).unwrap();
    allocator.free(Some(first)).unwrap();
    allocator.free(Some(second)).unwrap();
    [first, second]
}

fn assert_pair_slot(allocator: &MMgrOpt, pair: [StorageAddress; 2]) {
    let table = allocator.free_lists.active.as_ref().unwrap().as_slice();
    let offset = table[16..].as_ptr().addr() - table.as_ptr().addr();
    assert_table_row(&format!(
        "slot-head {offset} {} {}",
        u8::from(super::decode_slot(&table[16..24]) == pair[1].get() - 8),
        u8::from(allocator.space.read_word(pair[1].get() - 8) == Some(pair[0].get() - 8))
    ));
}

#[test]
fn actual_slots_drive_intrusive_lifo_pop_and_purge() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let pair = recycled_pair(&mut allocator);
    assert_pair_slot(&allocator, pair);
    let reused = allocator.allocate(16).unwrap();
    assert_table_row(&format!(
        "slot-pop {} {} {}",
        u8::from(reused == pair[1]),
        u8::from(allocator.free_lists.head(2).unwrap() == pair[0].get() - 8),
        u8::from(allocator.space.read_word(reused.get() - 8) == Some(16))
    ));
    allocator.free(Some(reused)).unwrap();
    let purged = allocator.purge(false);
    assert_table_row(&format!(
        "slot-purge {purged} {}",
        u8::from(allocator.free_lists.head(2).unwrap() == 0)
    ));
}

fn retained_distinct(slots: &NativeFreeListSlots) -> bool {
    let addresses: Vec<usize> = slots
        .retired
        .iter()
        .map(|table| table.as_slice().as_ptr().addr())
        .collect();
    let distinct: std::collections::HashSet<usize> = addresses.iter().copied().collect();
    assert!(!addresses.contains(&slots.active.as_ref().unwrap().as_slice().as_ptr().addr()));
    addresses.len() == distinct.len()
}

#[test]
fn repeated_initialize_retains_old_tables_without_head_authority() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let block = allocator.allocate(16).unwrap();
    allocator.free(Some(block)).unwrap();
    for _ in 0..3 {
        allocator.initialize_native_free_lists();
    }
    let slots = &allocator.free_lists;
    assert_table_row(&format!(
        "retained-tables {} {} {} {}",
        slots.retired.len(),
        u8::from(retained_distinct(slots)),
        u8::from(super::decode_slot(&slots.retired[0].as_slice()[16..24]) == block.get() - 8),
        u8::from(slots.head(2).unwrap() == 0)
    ));
    assert_eq!(allocator.purge(false), 0);
}

fn assert_null_table(threshold: usize, label: &str) {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, threshold);
    assert_table_row(&format!(
        "table-{label} {} {}",
        allocator.free_list_max,
        u8::from(allocator.free_lists.active.is_none())
    ));
    assert!(allocator.allocate(8).is_err());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| allocator.purge(false))).is_err()
    );
    allocator.threshold = 512;
    allocator.initialize_native_free_lists();
    let block = allocator.allocate(8).unwrap();
    allocator.free(Some(block)).unwrap();
}

#[test]
fn overflow_and_shared_backend_layout_refusal_are_distinct_null_table_states() {
    assert_null_table(usize::MAX - 7, "overflow");
    assert_null_table(isize::MAX as usize, "refused");
}

#[test]
fn null_table_free_guard_preserves_live_storage_for_recovery() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let block = allocator.allocate(24).unwrap();
    allocator.bytes_mut(block).unwrap().fill(7);
    allocator.threshold = usize::MAX - 7;
    allocator.initialize_native_free_lists();
    assert!(allocator.free(Some(block)).is_err());
    assert_eq!(allocator.bytes(block).unwrap(), [7; 24]);
    assert_eq!(allocator.free_lists.retired.len(), 1);
    allocator.threshold = 512;
    allocator.initialize_native_free_lists();
    allocator.free(Some(block)).unwrap();
}

fn node_identity(watched: &[usize], address: usize) -> usize {
    // Numeric labels compare independent native/Rust heaps without inventing pointer equality.
    watched
        .iter()
        .position(|node| *node == address)
        .map_or(0, |index| index + 1)
}

fn purge_nodes(allocator: &mut MMgrOpt) -> [StorageAddress; 4] {
    let nodes = [
        allocator.allocate(8).unwrap(),
        allocator.allocate(8).unwrap(),
        allocator.allocate(16).unwrap(),
        allocator.allocate(24).unwrap(),
    ];
    for index in [0, 1, 3, 2] {
        allocator.free(Some(nodes[index])).unwrap();
    }
    nodes
}

#[test]
fn purge_releases_ascending_classes_lifo_nodes_and_clears_heads_after_chain() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let nodes = purge_nodes(&mut allocator);
    let watched = nodes.map(|node| node.get() - 8);
    let mut events = Vec::new();
    let freed = allocator.release_medium_lists_observed(1, |index, node, head| {
        events.push(format!(
            "{index}:{}:{}",
            node_identity(&watched, node),
            node_identity(&watched, head)
        ));
    });
    assert_table_row(&format!("purge-order {freed} {}", events.join(",")));
    for node in watched {
        assert!(allocator.space.region_len(node).is_none());
    }
    assert!(allocator.free_lists.is_empty());
}

#[test]
fn clearing_a_slot_prefix_preserves_higher_canonical_heads() {
    let mut slots = NativeFreeListSlots::default();
    slots.replace_with_zeroed(3);
    slots.set_head(1, 11).unwrap();
    slots.set_head(3, 33).unwrap();
    slots.clear_through(2).unwrap();
    assert_eq!(slots.head(1).unwrap(), 0);
    assert_eq!(slots.head(3).unwrap(), 33);
    assert!(slots.validate_index(3).is_ok());
    assert!(slots.validate_index(usize::MAX).is_err());
}

fn preserved_traits<T: Send + Sync + Unpin>() {
    // Owned byte tables preserve the prior allocator's supported cross-thread auto traits.
    let _ = std::mem::size_of::<T>();
}

#[test]
fn canonical_table_preserves_allocator_send_sync_and_unpin() {
    preserved_traits::<NativeFreeListSlots>();
    preserved_traits::<MMgrOpt>();
}
