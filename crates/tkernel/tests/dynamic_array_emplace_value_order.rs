//! EmplaceValue reserves all destination blocks before running element constructors.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct NestedAllocation(usize);
struct DefaultAllocation(usize);
thread_local! {
    static ACTIVE_ALLOCATOR: RefCell<Option<BaseAllocator>> = const { RefCell::new(None) };
}

fn allocate_nested(allocator: &BaseAllocator) -> NestedAllocation {
    let allocation = allocator.allocate_raw(8).expect("nested allocation");
    NestedAllocation(allocation.as_ptr() as usize)
}

impl Default for DefaultAllocation {
    fn default() -> Self {
        let address = ACTIVE_ALLOCATOR.with(|slot| {
            slot.borrow()
                .as_ref()
                .expect("active incremental allocator")
                .allocate_raw(8)
                .expect("gap nested allocation")
                .as_ptr() as usize
        });
        Self(address)
    }
}

#[test]
fn emplace_value_allocates_the_target_block_before_its_constructor() {
    let allocator = BaseAllocator::incremental(4096);
    let mut array = DynamicArray::with_allocator(2, Some(allocator.clone()));
    array.append(NestedAllocation(0));
    let element = array
        .emplace_value(2, || allocate_nested(&allocator))
        .unwrap();
    let stored = element as *const NestedAllocation as usize;
    assert!(stored < element.0, "stored={stored}, nested={}", element.0);
}

#[test]
fn emplace_value_allocates_every_target_block_before_gap_constructors() {
    let allocator = BaseAllocator::incremental(4096);
    let mut array = DynamicArray::with_allocator(2, Some(allocator.clone()));
    array.append(DefaultAllocation(0));
    ACTIVE_ALLOCATOR.with(|slot| *slot.borrow_mut() = Some(allocator.clone()));
    let target = array
        .emplace_value(6, || DefaultAllocation(allocate_nested(&allocator).0))
        .unwrap() as *mut DefaultAllocation as usize;
    ACTIVE_ALLOCATOR.with(|slot| *slot.borrow_mut() = None);
    assert!(
        target < array[1].0,
        "target={target}, first_gap={}",
        array[1].0
    );
}

#[derive(Default)]
struct GapProbe;

#[test]
fn failed_requested_construction_keeps_the_old_logical_range() {
    let mut array = DynamicArray::<GapProbe>::with_increment(2);
    array.append(GapProbe);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        let _ = array.emplace_value(4, || panic!("requested constructor failure"));
    }));
    assert!(failure.is_err());
    assert_eq!(array.len(), 1);
    assert_eq!(array.iter().count(), 1);
}

static DROPPED: AtomicUsize = AtomicUsize::new(0);
struct DropProbe;
impl Default for DropProbe {
    fn default() -> Self {
        Self
    }
}
impl Drop for DropProbe {
    fn drop(&mut self) {
        DROPPED.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn failed_requested_construction_preserves_native_gap_destructor_behavior() {
    DROPPED.store(0, Ordering::SeqCst);
    let mut array = DynamicArray::<DropProbe>::with_increment(2);
    array.append(DropProbe);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        let _ = array.emplace_value(4, || panic!("requested constructor failure"));
    }));
    assert!(failure.is_err());
    assert_eq!(array.len(), 1);
    assert_eq!(DROPPED.load(Ordering::SeqCst), 0);
    drop(array);
    assert_eq!(DROPPED.load(Ordering::SeqCst), 1);
}
