//! Supplemental assignment return and copy-failure behavior beyond the legacy test declarations.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CopyBomb(i32);
static COPY_BOMB_INDEX: AtomicUsize = AtomicUsize::new(0);
static COPY_BOMB_AT: AtomicUsize = AtomicUsize::new(usize::MAX);

impl Clone for CopyBomb {
    fn clone(&self) -> Self {
        let index = COPY_BOMB_INDEX.fetch_add(1, Ordering::SeqCst) + 1;
        assert_ne!(
            index,
            COPY_BOMB_AT.load(Ordering::SeqCst),
            "selected copy failure"
        );
        Self(self.0)
    }
}

#[test]
fn assignment_adapters_return_self_and_preserve_allocator_modes() {
    let mut source = DynamicArray::with_allocator(2, Some(BaseAllocator::fresh_common()));
    source.append(10);
    source.append(20);
    let mut replacement = DynamicArray::with_allocator(2, Some(BaseAllocator::global_heap()));
    replacement.append(30);
    check_copy_assignment_returns(&source, &replacement);
}

fn check_copy_assignment_returns(source: &DynamicArray<i32>, replacement: &DynamicArray<i32>) {
    let mut copy = DynamicArray::with_allocator(4, Some(BaseAllocator::global_heap()));
    let destination = &mut copy as *mut DynamicArray<i32>;
    assert_eq!(copy.assign_returning(source, false) as *mut _, destination);
    assert_eq!(copy.allocator(), source.allocator());
    assert_eq!(
        copy.assign_default_returning(replacement) as *mut _,
        destination
    );
    assert_eq!(copy.allocator(), source.allocator());
    assert_eq!(
        copy.assign_operator_returning(replacement) as *mut _,
        destination
    );
    assert_eq!(copy.allocator(), replacement.allocator());
    copy.assign_returning(source, true)
        .assign_operator_returning(replacement);
    assert_eq!(copy.iter().copied().collect::<Vec<_>>(), [30]);
}

#[test]
fn move_assignment_returns_self_and_transfers_allocator() {
    let allocator = BaseAllocator::fresh_common();
    let mut source = DynamicArray::with_allocator(2, Some(allocator.clone()));
    source.append(10);
    source.append(20);
    let mut destination = DynamicArray::default();
    let destination_address = &mut destination as *mut DynamicArray<i32>;
    let returned = destination.move_from_returning(&mut source) as *mut _;
    assert_eq!(returned, destination_address);
    assert!(source.is_empty());
    assert_eq!(destination.allocator(), Some(&allocator));
    assert!(source.allocator().is_none());
    assert_eq!(destination.iter().copied().collect::<Vec<_>>(), [10, 20]);
}

#[test]
fn assignment_publishes_only_complete_blocks_after_clone_panics() {
    let mut source = DynamicArray::with_increment(2);
    for value in 1..=3 {
        source.append(CopyBomb(value));
    }
    let mut destination = DynamicArray::with_increment(2);
    destination.append(CopyBomb(99));
    assert_assignment_panic_size(&source, &mut destination, 2, 0);
    assert_assignment_panic_size(&source, &mut destination, 3, 2);
    assert_eq!(destination.value(0).unwrap().0, 1);
    assert_eq!(destination.value(1).unwrap().0, 2);
}

fn assert_assignment_panic_size(
    source: &DynamicArray<CopyBomb>,
    destination: &mut DynamicArray<CopyBomb>,
    panic_at: usize,
    expected_size: usize,
) {
    COPY_BOMB_INDEX.store(0, Ordering::SeqCst);
    COPY_BOMB_AT.store(panic_at, Ordering::SeqCst);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        destination.assign(source, true);
    }));
    assert!(result.is_err());
    assert_eq!(destination.len(), expected_size);
}
