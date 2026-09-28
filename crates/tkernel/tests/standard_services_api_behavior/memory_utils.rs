//! Standard_MemoryUtils: owning pointers backed by OCCT allocators.
use rustycad_tkernel::ncollection::{BaseAllocator, OccAllocator};
use rustycad_tkernel::standard::memory_utils::{
    make_oshared, make_oshared_in, make_shared, make_unique,
};
use std::sync::Arc;

#[test]
fn global_allocator_pointers_own_their_value() {
    let shared = make_shared(vec![1, 2]);
    let other = Arc::clone(&shared);
    assert_eq!(Arc::strong_count(&other), 2);
    assert_eq!(*make_unique(3.5), 3.5);
}

#[test]
fn oshared_values_live_in_the_given_allocator() {
    let arena = BaseAllocator::incremental(1024);
    let shared = make_oshared(&arena, 42u64);
    assert_eq!(**shared, 42);
    assert!(arena.reset(false).is_err(), "a live value pins the arena");
    drop(shared);
    arena.reset(false).unwrap();
}

#[test]
fn typed_allocator_overloads_share_one_value() {
    let arena = BaseAllocator::incremental(1024);
    let shared = make_oshared_in(&OccAllocator::<i32>::new(arena.clone()), -7);
    let copy = shared.clone();
    assert_eq!((*copy, copy.use_count()), (-7, 2));
    assert!(arena.reset(false).is_err());
    drop((shared, copy));
    arena.reset(false).unwrap();
}
