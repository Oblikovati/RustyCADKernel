//! Rust-only alignment and panic/concurrency invariants around native raw allocation policy.
#![allow(unsafe_code)]
use allocator_api2::alloc::{Allocator, Layout};
use rustycad_tkernel::ncollection::{AllocatedBox, BaseAllocator};
use std::sync::{Arc, Barrier};

#[test]
fn every_cursor_residue_reserves_only_checked_required_padding() {
    for align in [8, 16, 32, 64, 256] {
        for prefix in (0..align).step_by(4) {
            assert_cursor_geometry(align, prefix);
        }
    }
}
fn assert_cursor_geometry(align: usize, prefix: usize) {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let head = (prefix != 0).then(|| arena.allocate(prefix).unwrap());
    let before = arena.accumulating_chain()[0];
    let layout = Layout::from_size_align(align, align).unwrap();
    let pointer = Allocator::allocate(&arena, layout).unwrap().cast::<u8>();
    let after = arena.accumulating_chain()[0];
    let needed = align + (before.address + before.free_size - align) % align;
    assert_eq!(before.free_size - after.free_size, needed);
    assert_eq!(pointer.as_ptr().addr(), after.address + after.free_size);
    assert_eq!(pointer.as_ptr().addr() % align, 0);
    assert_eq!(after.alloc_count, 1 + i32::from(prefix != 0));
    // SAFETY: original live layout/pointer; the prefix owner remains live until afterward.
    unsafe { Allocator::deallocate(&arena, pointer, layout) };
    drop(head);
    assert_eq!(arena.accumulating_chain()[0].alloc_count, 0);
}
#[repr(align(256))]
struct Aligned256([u64; 32]);
#[test]
fn over_aligned_payloads_survive_fresh_blocks_and_arbitrary_drop_order() {
    let arena = BaseAllocator::accumulating(512).unwrap();
    let mut owners: Vec<_> = (0..8)
        .map(|i| arena.allocate_value(Aligned256([i; 32])))
        .collect();
    for (index, value) in owners.iter().enumerate() {
        assert_eq!(value.0, [index as u64; 32]);
        assert_eq!((&**value as *const Aligned256).addr() % 256, 0);
    }
    owners.swap_remove(3);
    owners.swap_remove(0);
    drop(owners);
    assert_eq!(arena.accumulating_chain().len(), 1);
    assert_eq!(arena.accumulating_chain()[0].free_size, 512);
}
#[test]
fn constructor_panic_releases_partial_array_and_restores_sole_block() {
    let arena = BaseAllocator::accumulating(1024).unwrap();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        arena.allocate_array(8, |index| {
            assert_ne!(index, 3, "test constructor failure at index 3");
            [index as u64; 4]
        })
    }));
    assert!(caught.is_err());
    let chain = arena.accumulating_chain();
    assert_eq!(
        (chain.len(), chain[0].alloc_count, chain[0].free_size),
        (1, 0, 1024)
    );
}
fn parallel_owners(
    arena: BaseAllocator,
    barrier: Arc<Barrier>,
    value: u64,
) -> Vec<AllocatedBox<[u64; 4]>> {
    barrier.wait();
    (0..8).map(|_| arena.allocate_value([value; 4])).collect()
}
#[test]
fn allocation_and_counts_remain_atomic_across_shared_threads() {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let barrier = Arc::new(Barrier::new(4));
    let jobs: Vec<_> = (0..4)
        .map(|value| {
            let (arena, barrier) = (arena.clone(), barrier.clone());
            std::thread::spawn(move || parallel_owners(arena, barrier, value))
        })
        .collect();
    let owners: Vec<_> = jobs.into_iter().map(|job| job.join().unwrap()).collect();
    assert_eq!(arena.accumulating_chain()[0].alloc_count, 32);
    for (index, group) in owners.iter().enumerate() {
        assert!(group.iter().all(|value| **value == [index as u64; 4]));
    }
    drop(owners);
    assert_eq!(arena.accumulating_chain()[0].alloc_count, 0);
}
