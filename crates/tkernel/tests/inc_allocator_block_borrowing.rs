//! Baseline ownership contracts must survive replacing arena bookkeeping with actual headers.
use rustycad_tkernel::ncollection::{AllocatedArray, AllocatedBox, AllocatedBytes, BaseAllocator};
use std::cell::Cell;

fn assert_owner_traits<T: Send + Sync + Unpin>() {
    // A type-level bound checks the owner without constructing or touching its payload.
    let _: std::marker::PhantomData<T> = std::marker::PhantomData;
}

#[test]
fn allocator_and_sendable_owners_preserve_auto_traits() {
    assert_owner_traits::<BaseAllocator>();
    assert_owner_traits::<AllocatedBytes>();
    assert_owner_traits::<AllocatedBox<&u64>>();
    assert_owner_traits::<AllocatedArray<&u64>>();
}

fn shorten_box<'long: 'short, 'short>(
    owner: AllocatedBox<&'long u64>,
    _scope: &'short u64,
) -> AllocatedBox<&'short u64> {
    owner
}

fn shorten_array<'long: 'short, 'short>(
    owner: AllocatedArray<&'long u64>,
    _scope: &'short u64,
) -> AllocatedArray<&'short u64> {
    owner
}

#[test]
fn borrowed_payload_owners_remain_covariant() {
    let arena = BaseAllocator::incremental(2048);
    let payload = 41;
    let scope = 7;
    let boxed = shorten_box(arena.allocate_value(&payload), &scope);
    let array = shorten_array(arena.allocate_array(2, |_| &payload), &scope);
    assert_eq!(**boxed, 41);
    assert_eq!(array.as_slice(), &[&41, &41]);
}

struct BorrowedDrop<'a>(&'a Cell<usize>);
impl Drop for BorrowedDrop<'_> {
    fn drop(&mut self) {
        let prior = self.0.get();
        self.0.set(prior + 1);
    }
}

#[test]
fn external_borrowed_destructors_need_no_static_payload_bound() {
    let drops = Cell::new(0);
    let arena = BaseAllocator::incremental(2048);
    let boxed = arena.allocate_value(BorrowedDrop(&drops));
    let array = arena.allocate_array(2, |_| BorrowedDrop(&drops));
    drop(arena);
    drop(boxed);
    assert_eq!(drops.get(), 1);
    drop(array);
    assert_eq!(drops.get(), 3);
}

#[test]
fn live_mutable_payloads_survive_sibling_allocation_and_header_updates() {
    let arena = BaseAllocator::incremental(2048);
    let mut first = arena.allocate_value(11_u64);
    let retained = &mut *first;
    for index in 0..12 {
        let mut sibling = arena.allocate_value([index as u64; 128]);
        let current = &mut sibling[3];
        *retained += 1;
        *current += *retained;
        assert_eq!(*current, index as u64 + *retained);
    }
    assert_eq!(*retained, 23);
    assert!(arena.reset(false).is_err());
    drop(first);
    arena.reset(true).unwrap();
}
