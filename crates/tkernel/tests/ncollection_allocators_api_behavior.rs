//! Allocation policies of NCollection_BaseAllocator and its Inc, Acc, Heap, Aligned subclasses.
// The raw Free mirrors the native pointer API, which Rust marks unsafe.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    ACC_DEFAULT_BLOCK_SIZE, AccAlignedPtr, AccAlignedSize, BaseAllocator, INC_DEFAULT_BLOCK_SIZE,
    IncBlock, IncBlockSizeLevel,
};
use std::ptr::NonNull;

fn address(pointer: NonNull<u8>) -> usize {
    pointer.as_ptr() as usize
}

fn spawn_incremental_worker(
    allocator: std::sync::Arc<BaseAllocator>,
    allocations: usize,
) -> std::thread::JoinHandle<Vec<usize>> {
    std::thread::spawn(move || {
        (0..allocations)
            .map(|_| address(allocator.allocate_raw(64).unwrap()))
            .collect()
    })
}

#[test]
fn block_size_levels_grow_like_increase_block_size() {
    assert_eq!(IncBlockSizeLevel::of(2048), IncBlockSizeLevel::Min);
    assert_eq!(IncBlockSizeLevel::of(12 * 1024), IncBlockSizeLevel::Small);
    assert_eq!(IncBlockSizeLevel::of(196 * 1024), IncBlockSizeLevel::Medium);
    assert_eq!(IncBlockSizeLevel::of(786 * 1024), IncBlockSizeLevel::Large);
    assert_eq!(
        IncBlockSizeLevel::of(12 * 1024 * 1024),
        IncBlockSizeLevel::Max
    );
    assert_eq!(IncBlockSizeLevel::Min.grow(2048), 16384);
    assert_eq!(IncBlockSizeLevel::Small.grow(12288), 49152);
    assert_eq!(IncBlockSizeLevel::Medium.grow(200_000), 400_000);
    assert_eq!(IncBlockSizeLevel::Large.grow(1_000_001), 1_500_002); // lround(1500001.5)
    assert_eq!(IncBlockSizeLevel::Max.grow(7), 7);
    assert_eq!(
        IncBlock::new(64),
        IncBlock {
            cursor: 0,
            available_size: 64
        }
    );
}

#[test]
fn incremental_bumps_contiguously_and_small_sizes_select_the_default_block() {
    let allocator = BaseAllocator::incremental(100); // below the 2 KB minimum: 12 KB blocks
    let first = allocator.allocate_raw(10).unwrap();
    let second = allocator.allocate_raw(3).unwrap();
    let third = allocator.allocate_raw(0).unwrap();
    assert_eq!(address(second), address(first) + 10); // no alignment padding upstream
    assert_eq!(address(third), address(second) + 3);
    let rest = allocator.allocate_raw(INC_DEFAULT_BLOCK_SIZE - 13).unwrap();
    assert_eq!(address(rest), address(first) + 13);
    let next = allocator.allocate_raw(1).unwrap();
    assert_ne!(address(next), address(rest) + INC_DEFAULT_BLOCK_SIZE - 13);
}

#[test]
fn every_fifth_incremental_block_grows() {
    let allocator = BaseAllocator::incremental(2048);
    let starts: Vec<usize> = (0..5)
        .map(|_| address(allocator.allocate_raw(2048).unwrap()))
        .collect();
    // The fifth block is 2048 * 8 bytes, so it keeps room for more 2 KB requests.
    let inside_fifth = address(allocator.allocate_raw(2048).unwrap());
    assert_eq!(inside_fifth, starts[4] + 2048);
    let more = address(allocator.allocate_raw(12 * 1024).unwrap());
    assert_eq!(more, starts[4] + 4096);
}

#[test]
fn incremental_reset_rewinds_newest_block_first_and_release_forgets_block_size() {
    let allocator = BaseAllocator::incremental(4096);
    let first = address(allocator.allocate_raw(4096).unwrap());
    let second = address(allocator.allocate_raw(4096).unwrap());
    assert_ne!(second, first + 4096);
    allocator.reset(false).unwrap();
    assert_eq!(address(allocator.allocate_raw(8).unwrap()), second);
    allocator.reset(true).unwrap();
    // clean() restores THE_DEFAULT_BLOCK_SIZE, so two 4 KB requests now share a 12 KB block.
    let a = address(allocator.allocate_raw(4096).unwrap());
    assert_eq!(address(allocator.allocate_raw(4096).unwrap()), a + 4096);
}

#[test]
fn incremental_free_is_a_noop_and_reset_refuses_live_owners() {
    let allocator = BaseAllocator::incremental(0);
    let raw = allocator.allocate_raw(16).unwrap();
    // SAFETY: raw comes from this allocator; the incremental Free does nothing.
    unsafe { allocator.free_raw(raw) }.unwrap();
    let owned = allocator.allocate(24).unwrap();
    assert!(owned.as_slice().iter().all(|byte| *byte == 0));
    let failure = allocator.reset(false).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(failure.message().starts_with("1 live allocations"));
    drop(owned);
    allocator.reset(false).unwrap();
}

#[test]
fn nullable_free_matches_native_null_behavior_for_every_allocator_kind() {
    unsafe { BaseAllocator::common().free_raw_nullable(std::ptr::null_mut()) }.unwrap();
    unsafe { BaseAllocator::global_heap().free_raw_nullable(std::ptr::null_mut()) }.unwrap();
    unsafe { BaseAllocator::incremental(0).free_raw_nullable(std::ptr::null_mut()) }.unwrap();
    unsafe { BaseAllocator::aligned(16).free_raw_nullable(std::ptr::null_mut()) }.unwrap();
    let accumulating = BaseAllocator::accumulating(64).unwrap();
    let failure = unsafe { accumulating.free_raw_nullable(std::ptr::null_mut()) }.unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(
        failure
            .message()
            .ends_with("Trying to free an invalid address")
    );
    let raw = accumulating.allocate_raw(8).unwrap();
    unsafe { accumulating.free_raw(raw) }.unwrap();
}

#[test]
fn accumulating_protected_alignment_helpers_match_native_arithmetic() {
    assert_acc_aligned_size_rounding();
    assert_acc_aligned_pointer_views();
    assert_acc_aligned_pointer_arithmetic();
}

fn assert_acc_aligned_size_rounding() {
    assert_eq!(
        [0, 1, 2, 3, 4, 5, usize::MAX].map(|size| AccAlignedSize::new(size).get()),
        [0, 4, 4, 4, 4, 8, 0]
    );
}

fn assert_acc_aligned_pointer_views() {
    let mut storage = AlignedBytes([0; 16]);
    let base = storage.0.as_mut_ptr();
    let raw = base.wrapping_add(3);
    let aligned = AccAlignedPtr::from_ptr(raw);
    assert_eq!(aligned.as_ptr(), base);
    assert_eq!(AccAlignedPtr::default().as_ptr(), std::ptr::null_mut());
}

fn assert_acc_aligned_pointer_arithmetic() {
    let mut storage = AlignedBytes([0; 16]);
    let base = storage.0.as_mut_ptr();
    let aligned = AccAlignedPtr::from_ptr(base);
    let size = AccAlignedSize::new(1);
    assert_eq!((aligned + size).as_ptr(), base.wrapping_add(4));
    assert_eq!(((aligned + size) - size).as_ptr(), base);
    let mut moved = aligned;
    assert_eq!(
        moved.wrapping_add_assign(size).as_ptr(),
        base.wrapping_add(4)
    );
    assert_eq!(moved.wrapping_sub_assign(size).as_ptr(), base);
    moved += size;
    moved += size;
    moved -= size;
    assert_eq!(moved.as_ptr(), base.wrapping_add(4));
}

#[repr(align(4))]
struct AlignedBytes([u8; 16]);

#[test]
fn incremental_thread_safety_request_keeps_parallel_allocations_unique() {
    use std::sync::Arc;

    const THREADS: usize = 4;
    const PER_THREAD: usize = 128;
    let allocator = Arc::new(BaseAllocator::incremental(4096));
    allocator.set_thread_safe(true).unwrap();
    let addresses = allocate_parallel(Arc::clone(&allocator), THREADS, PER_THREAD);
    assert_eq!(addresses.len(), THREADS * PER_THREAD);
    allocator.set_thread_safe(false).unwrap();
    allocator.reset(false).unwrap();
    assert!(allocator.allocate_raw(1).is_some());
}

fn allocate_parallel(
    allocator: std::sync::Arc<BaseAllocator>,
    thread_count: usize,
    allocations_per_thread: usize,
) -> Vec<usize> {
    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            spawn_incremental_worker(std::sync::Arc::clone(&allocator), allocations_per_thread)
        })
        .collect();
    let mut addresses: Vec<usize> = handles
        .into_iter()
        .flat_map(|handle| handle.join().unwrap())
        .collect();
    addresses.sort_unstable();
    addresses.dedup();
    addresses
}

#[test]
fn thread_safety_request_rejects_non_incremental_allocator() {
    let allocators = [
        (BaseAllocator::common(), "NCollection_BaseAllocator"),
        (BaseAllocator::global_heap(), "NCollection_HeapAllocator"),
        (
            BaseAllocator::accumulating(64).unwrap(),
            "NCollection_AccAllocator",
        ),
        (BaseAllocator::aligned(16), "NCollection_AlignedAllocator"),
    ];
    for (allocator, actual) in allocators {
        let error = allocator.set_thread_safe(true).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_TypeMismatch");
        assert!(error.message().contains(actual));
        assert!(error.message().contains("NCollection_IncAllocator"));
    }
}

#[test]
fn accumulating_allocations_descend_rounded_to_four_bytes() {
    let allocator = BaseAllocator::accumulating(64).unwrap();
    let chain = allocator.accumulating_chain();
    assert_eq!(chain.len(), 1);
    assert_eq!((chain[0].free_size, chain[0].alloc_count), (64, 0));
    let first = allocator.allocate_raw(5).unwrap();
    assert_eq!(address(first), chain[0].address + 56);
    let second = allocator.allocate_raw(0).unwrap(); // zero still counts as an allocation
    assert_eq!(second, first);
    let state = allocator.accumulating_block(address(first)).unwrap();
    assert_eq!((state.free_size, state.alloc_count), (56, 2));
}

#[test]
fn accumulating_oversize_requests_get_a_dedicated_block_and_empty_blocks_are_freed() {
    let allocator = BaseAllocator::accumulating(64).unwrap();
    let small = allocator.allocate_raw(8).unwrap();
    let big = allocator.allocate_raw(100).unwrap();
    let chain = allocator.accumulating_chain();
    assert_eq!(chain.len(), 2);
    assert_eq!((chain[0].address, chain[0].free_size), (address(big), 0));
    // SAFETY: big is a live allocation of this allocator.
    unsafe { allocator.free_raw(big) }.unwrap();
    assert_eq!(allocator.accumulating_chain().len(), 1);
    // SAFETY: small is a live allocation of this allocator.
    unsafe { allocator.free_raw(small) }.unwrap();
    let sole = allocator.accumulating_chain();
    assert_eq!(
        (sole.len(), sole[0].free_size, sole[0].alloc_count),
        (1, 64, 0)
    );
}

#[test]
fn accumulating_free_rejects_foreign_addresses_and_empty_blocks() {
    let allocator = BaseAllocator::accumulating(ACC_DEFAULT_BLOCK_SIZE).unwrap();
    let local = 0_u64;
    let foreign = NonNull::from(&local).cast::<u8>();
    // SAFETY: a foreign address is detected before any block is touched.
    let failure = unsafe { allocator.free_raw(foreign) }.unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(
        failure
            .message()
            .ends_with("Trying to free an invalid address")
    );
    let raw = allocator.allocate_raw(4).unwrap();
    // SAFETY: raw is live; the second free hits an empty block and is refused.
    unsafe { allocator.free_raw(raw) }.unwrap();
    assert!(unsafe { allocator.free_raw(raw) }.is_err());
    assert!(BaseAllocator::accumulating(0).is_err());
}

#[test]
fn accumulating_scans_previous_blocks_before_creating_one() {
    let allocator = BaseAllocator::accumulating(64).unwrap();
    let _fill = allocator.allocate_raw(40).unwrap(); // first block keeps 24 bytes
    let _next = allocator.allocate_raw(40).unwrap(); // second block, 24 left
    let _full = allocator.allocate_raw(24).unwrap(); // fills the second block
    let reused = allocator.allocate_raw(20).unwrap(); // found in the first block
    let chain = allocator.accumulating_chain();
    assert_eq!(chain.len(), 2);
    assert_eq!(address(reused), chain[1].address + 4);
}

#[test]
fn aligned_allocator_follows_mm_malloc() {
    for alignment in [0_usize, 3, 12, 24] {
        assert!(BaseAllocator::aligned(alignment).allocate_raw(16).is_none());
    }
    for alignment in [1_usize, 2, 4, 8, 16, 64, 4096] {
        let allocator = BaseAllocator::aligned(alignment);
        let raw = allocator.allocate_optimal_raw(100).unwrap();
        assert_eq!(address(raw) % alignment, 0);
        // SAFETY: raw is a live allocation of this allocator.
        unsafe { allocator.free_raw(raw) }.unwrap();
    }
}

#[test]
fn heap_and_common_allocators_are_distinct_singletons() {
    let heap = BaseAllocator::global_heap();
    assert_eq!(heap, BaseAllocator::global_heap());
    assert_ne!(heap, BaseAllocator::common());
    for allocator in [heap, BaseAllocator::common()] {
        let raw = allocator.allocate_raw(3).unwrap();
        assert_eq!(address(raw) % 16, 0);
        // SAFETY: raw is a live allocation of this allocator.
        unsafe { allocator.free_raw(raw) }.unwrap();
    }
}

#[test]
fn arenas_serve_over_aligned_typed_values() {
    #[repr(align(64))]
    struct Wide(u8);
    for allocator in [
        BaseAllocator::incremental(0),
        BaseAllocator::accumulating(256).unwrap(),
    ] {
        let values: Vec<_> = (0..8).map(|i| allocator.allocate_value(Wide(i))).collect();
        for (index, value) in values.iter().enumerate() {
            assert_eq!((&**value as *const Wide as usize) % 64, 0);
            assert_eq!(value.0 as usize, index);
        }
    }
}
