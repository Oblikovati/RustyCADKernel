//! Ownership and alignment requirements of the allocator adapter.
use rustycad_tkernel::ncollection::BaseAllocator;
#[repr(align(256))]
struct Aligned(u64);
#[test]
fn incremental_allocations_are_aligned_and_retained_until_reset() {
    let arena = BaseAllocator::incremental(128);
    let mut first = arena.allocate_value(Aligned(42));
    let second = arena.allocate_value(Aligned(99));
    assert_eq!((&*first as *const Aligned as usize) % 256, 0);
    assert_eq!((&*second as *const Aligned as usize) % 256, 0);
    assert_ne!(&*first as *const Aligned, &*second as *const Aligned);
    first.0 = 17;
    assert_eq!(second.0, 99);
    assert!(arena.reset(false).is_err());
    drop(first);
    drop(second);
    arena.reset(false).unwrap();
    let value = arena.allocate_value(Aligned(123));
    assert_eq!(value.0, 123);
    drop(value);
    arena.reset(true).unwrap();
}
#[test]
fn base_allocator_identity_and_bytes() {
    let common = BaseAllocator::common();
    assert_eq!(common, BaseAllocator::common());
    let mut bytes = common.allocate(1024).expect("calloc succeeds");
    bytes.as_mut_slice().fill(0xAB);
    assert!(bytes.as_slice().iter().all(|byte| *byte == 0xAB));
    assert_ne!(common, BaseAllocator::incremental(128));
}
#[test]
fn buffer_allocation_state_and_allocator_changes() {
    use rustycad_tkernel::ncollection::Buffer;
    let allocator = BaseAllocator::incremental(128);
    let mut buffer = Buffer::new(Some(allocator.clone()), 0).unwrap();
    assert!(!buffer.is_empty());
    assert_eq!(buffer.size(), 0);
    assert_eq!(buffer.allocator(), Some(&allocator));
    assert!(buffer.allocate(64).unwrap());
    buffer.as_mut_slice().fill(17);
    assert_eq!(buffer.as_slice(), &[17; 64]);
    buffer.free().unwrap();
    assert!(buffer.is_empty());
    assert_eq!(buffer.size(), 0);
    buffer.set_allocator(None).unwrap();
    assert!(buffer.allocator().is_none());
    assert!(!buffer.allocate(0).unwrap());
    assert!(buffer.is_empty());
    allocator.reset(true).unwrap();
}
#[test]
fn allocated_array_is_contiguous_mutable_and_drops_elements() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Tracked(Arc<AtomicUsize>);
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let allocator = BaseAllocator::incremental(128);
    let dropped = Arc::new(AtomicUsize::new(0));
    let mut array = allocator.allocate_array(5, |_| Tracked(dropped.clone()));
    array.as_mut_slice().reverse();
    assert_eq!(array.as_slice().len(), 5);
    assert!(allocator.reset(false).is_err());
    drop(array);
    assert_eq!(dropped.load(Ordering::SeqCst), 5);
    allocator.reset(false).unwrap();
}
