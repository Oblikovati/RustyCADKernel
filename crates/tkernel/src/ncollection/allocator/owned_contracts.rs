use super::*;
use std::cell::Cell;
struct Dropped<'a>(&'a Cell<usize>);
impl Drop for Dropped<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[test]
fn borrowed_construction_keeps_actual_allocator_reference_counts() {
    let arena = BaseAllocator::incremental(1024);
    let owner = arena.allocate_value_with(|| {
        assert_eq!(arena.ref_count(), 1);
        7
    });
    assert_eq!(arena.ref_count(), 2);
    assert_eq!(*owner, 7);
    drop(owner);
    assert_eq!(arena.ref_count(), 1);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn construction_panic_releases_atomic_typed_lease_without_extra_handle() {
    let arena = BaseAllocator::incremental(1024);
    let failure = std::panic::catch_unwind(|| {
        arena.allocate_value_with(|| {
            assert_eq!(arena.ref_count(), 1);
            assert!(arena.reset(false).is_err());
            panic!("rejected constructor; expected a produced value");
        })
    });
    assert!(failure.is_err());
    assert_eq!(arena.ref_count(), 1);
    assert!(arena.reset(false).is_ok());
}
fn abandon<'a>(arena: &BaseAllocator, drops: &'a Cell<usize>) -> NonNull<Dropped<'a>> {
    let owner = arena.allocate_value_with(|| Dropped(drops));
    let (pointer, resource) = owner.into_raw_with_allocator();
    drop(resource);
    // SAFETY: no reference/owner remains and this allocation is relinquished exactly once.
    unsafe {
        arena.relinquish_typed_to_raw();
    }
    NonNull::new(pointer).unwrap()
}
#[test]
fn re_adoption_restores_incremental_lease_before_safe_owner_returns() {
    let arena = BaseAllocator::incremental(1024);
    let drops = Cell::new(0);
    let pointer = abandon(&arena, &drops);
    assert_eq!(drops.get(), 0);
    // SAFETY: initialized exact type/layout remains live; this thread excludes reset/free.
    let owner = unsafe { arena.adopt_abandoned_value(pointer) };
    assert!(arena.reset(false).is_err());
    assert_eq!(arena.ref_count(), 2);
    drop(owner);
    assert_eq!(drops.get(), 1);
    assert!(arena.reset(false).is_ok());
}
fn active_allocations(arena: &BaseAllocator) -> i32 {
    arena
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum()
}
#[test]
fn accumulating_handoff_and_re_adoption_leave_native_allocation_count_intact() {
    let arena = BaseAllocator::accumulating(1024).unwrap();
    let drops = Cell::new(0);
    let pointer = abandon(&arena, &drops);
    assert_eq!(active_allocations(&arena), 1);
    // SAFETY: same initialized allocation was abandoned; no reset/free or competing owner exists.
    let owner = unsafe { arena.adopt_abandoned_value(pointer) };
    assert_eq!(active_allocations(&arena), 1);
    drop(owner);
    assert_eq!(drops.get(), 1);
    assert_eq!(active_allocations(&arena), 0);
}
#[test]
fn reset_reclaims_abandoned_incremental_storage_without_running_payload_drop() {
    let arena = BaseAllocator::incremental(1024);
    let drops = Cell::new(0);
    let pointer = abandon(&arena, &drops);
    assert!(arena.reset(false).is_ok());
    let replacement = arena.allocate_value(11_u8);
    assert_eq!(
        std::ptr::from_ref(&*replacement).cast::<u8>(),
        pointer.as_ptr().cast::<u8>()
    );
    assert_eq!(drops.get(), 0);
    drop(replacement);
    assert!(arena.reset(true).is_ok());
}
