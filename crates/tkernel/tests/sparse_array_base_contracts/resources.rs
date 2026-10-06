//! Injected resources retain alignment and reset leases for native sparse allocations.
use super::fixture::{discard_integer, insert_integer};
use rustycad_tkernel::ncollection::{BaseAllocator, SparseArrayBase};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};

pub(super) fn allocator_live_slots(resource: &BaseAllocator) -> i32 {
    resource
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum()
}

#[test]
fn accumulating_sparse_headers_are_aligned_before_publication() {
    let resource = BaseAllocator::accumulating(256).unwrap();
    let base = SparseArrayBase::with_allocator(4, 4, discard_integer, resource.clone());
    insert_integer(&base, 0, 10);
    assert_eq!(base.block_address(0).addr() % align_of::<usize>(), 0);
    assert_eq!(unsafe { *base.get_value(0).unwrap().cast::<i32>() }, 10);
    assert_eq!(allocator_live_slots(&resource), 2);
    drop(base);
    assert_eq!(allocator_live_slots(&resource), 0);
}

#[test]
fn incremental_sparse_allocations_prevent_reset_until_released() {
    let resource = BaseAllocator::incremental(256);
    let base = SparseArrayBase::with_allocator(4, 4, discard_integer, resource.clone());
    insert_integer(&base, 0, 10);
    assert!(resource.reset(false).is_err());
    assert!(resource.reset(true).is_err());
    assert_eq!(unsafe { *base.get_value(0).unwrap().cast::<i32>() }, 10);
    drop(base);
    assert!(resource.reset(false).is_ok());
    assert!(resource.reset(true).is_ok());
}

#[test]
fn failed_construction_releases_resources_without_destroying_unpublished_items() {
    let resource = BaseAllocator::accumulating(256).unwrap();
    let base = SparseArrayBase::with_allocator(4, 4, discard_integer, resource.clone());
    let creations = Cell::new(0);
    assert!(construction_panics(&base, &creations));
    assert_eq!(
        (creations.get(), base.size(), base.has_value(0)),
        (1, 0, false)
    );
    let mut destructions = 0;
    unsafe { base.clear_with(|_| destructions += 1) };
    assert_eq!((destructions, allocator_live_slots(&resource)), (0, 0));
}

fn construction_panics(base: &SparseArrayBase, creations: &Cell<usize>) -> bool {
    catch_unwind(AssertUnwindSafe(|| unsafe {
        base.set_value_with(
            0,
            std::ptr::null_mut(),
            |_, _| {
                creations.set(creations.get() + 1);
                panic!("injected constructor failure");
            },
            |_, _| panic!("unexpected assignment"),
        );
    }))
    .is_err()
}

#[test]
fn exchanged_incremental_leases_follow_live_allocations() {
    let first_resource = BaseAllocator::incremental(256);
    let second_resource = BaseAllocator::incremental(256);
    let mut first = SparseArrayBase::with_allocator(4, 4, discard_integer, first_resource.clone());
    let mut second =
        SparseArrayBase::with_allocator(4, 9, discard_integer, second_resource.clone());
    insert_integer(&first, 0, 10);
    insert_integer(&second, 8, 80);
    unsafe { first.exchange(&mut second) };
    first.exchange_self();
    assert!(first_resource.reset(false).is_err());
    assert!(second_resource.reset(false).is_err());
    drop(first);
    assert!(second_resource.reset(true).is_ok());
    assert!(first_resource.reset(true).is_err());
    drop(second);
    assert!(first_resource.reset(true).is_ok());
}

#[test]
fn failed_allocation_keeps_table_and_callbacks_unpublished() {
    let base = SparseArrayBase::with_allocator(4, 4, discard_integer, BaseAllocator::aligned(3));
    let creations = Cell::new(0);
    assert!(construction_panics(&base, &creations));
    assert_eq!(
        (creations.get(), base.size(), base.block_slots()),
        (0, 0, 0)
    );
    assert!(base.block_address(0).is_null());
    let mut destructions = 0;
    unsafe { base.clear_with(|_| destructions += 1) };
    assert_eq!(destructions, 0);
}

#[test]
fn incremental_growth_releases_replaced_tables_and_removed_blocks() {
    let resource = BaseAllocator::incremental(256);
    let base = SparseArrayBase::with_allocator(4, 4, discard_integer, resource.clone());
    insert_integer(&base, 0, 10);
    insert_integer(&base, 100, 100);
    assert!(resource.reset(false).is_err());
    unsafe { base.unset_value(0, discard_integer) };
    unsafe { base.unset_value(100, discard_integer) };
    assert!(resource.reset(false).is_err());
    unsafe { base.clear_items(discard_integer) };
    assert!(resource.reset(true).is_ok());
}

#[test]
fn failed_construction_retains_incremental_lease_until_empty_block_cleanup() {
    let resource = BaseAllocator::incremental(256);
    let base = SparseArrayBase::with_allocator(4, 4, discard_integer, resource.clone());
    assert!(construction_panics(&base, &Cell::new(0)));
    assert_eq!(base.size(), 0);
    assert!(resource.reset(false).is_err());
    unsafe { base.clear_items(discard_integer) };
    assert!(resource.reset(true).is_ok());
}
