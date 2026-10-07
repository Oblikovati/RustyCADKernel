use super::*;
use std::cell::Cell;
struct Counted<'a>(&'a Cell<usize>);
impl Drop for Counted<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn resources() -> [BaseAllocator; 5] {
    [
        BaseAllocator::common(),
        BaseAllocator::global_heap(),
        BaseAllocator::incremental(1024),
        BaseAllocator::accumulating(1024).unwrap(),
        BaseAllocator::aligned(32),
    ]
}
fn abandon_initialized<'a>(
    allocator: &BaseAllocator,
    drops: &'a Cell<usize>,
) -> NonNull<Counted<'a>> {
    let pair = TreePairAllocation::new_native_with(allocator, |_| Counted(drops));
    let pointer = NonNull::new(pair.pointer(0)).unwrap();
    let failed = catch_unwind(AssertUnwindSafe(|| {
        pair.edit_native(|_| panic!("rejected pair assignment; expected permitted assignment"));
    }));
    assert!(failed.is_err());
    assert_eq!(drops.get(), 0);
    pointer
}
fn reclaim_initialized(allocator: &BaseAllocator, pointer: NonNull<Counted<'_>>) {
    // SAFETY: exact unique abandoned pair from abandon_initialized; no reset/free intervenes.
    let allocation =
        unsafe { allocator.adopt_abandoned_value(pointer.cast::<MaybeUninit<[Counted<'_>; 2]>>()) };
    for index in 0..2 {
        // SAFETY: both completed payloads survive native failure, with no competing owner.
        unsafe { std::ptr::drop_in_place(pointer.as_ptr().add(index)) };
    }
    drop(allocation);
}
#[test]
fn failed_native_assignment_can_be_typed_repaired_under_every_allocator_policy() {
    for allocator in resources() {
        let drops = Cell::new(0);
        let pointer = abandon_initialized(&allocator, &drops);
        reclaim_initialized(&allocator, pointer);
        assert_eq!(drops.get(), 2);
        allocator.reset(true).unwrap();
    }
}
#[test]
fn failed_zero_sized_pair_does_not_release_an_unrelated_incremental_lease() {
    let allocator = BaseAllocator::incremental(1024);
    let retained = allocator.allocate_value(42);
    let pair = TreePairAllocation::new_native_with(&allocator, |_| ());
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            pair.edit_native(|_| panic!("rejected zero-sized pair; expected permitted assignment"));
        }))
        .is_err()
    );
    assert!(allocator.reset(true).is_err());
    drop(retained);
    allocator.reset(true).unwrap();
}
#[test]
fn broader_owned_constructor_failure_still_destroys_completed_children() {
    let allocator = BaseAllocator::incremental(1024);
    let drops = Cell::new(0);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            TreePairAllocation::new_with(&allocator, |index| {
                assert_eq!(
                    index, 0,
                    "slot {index}; expected first slot before forced failure"
                );
                Counted(&drops)
            });
        }))
        .is_err()
    );
    assert_eq!(drops.get(), 1);
    allocator.reset(true).unwrap();
}
