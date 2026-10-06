//! The public Standard owners use the exact common raw allocation family, not an independent heap.
use super::StandardByteStorage;
use crate::ncollection::BaseAllocator;
use std::mem::ManuallyDrop;

#[test]
fn standard_and_common_allocations_share_the_exact_free_backend() {
    let standard = ManuallyDrop::new(StandardByteStorage::allocate(3, 16, true).unwrap());
    // SAFETY: Standard's retained pointer comes from the identical common raw allocation backend.
    unsafe { BaseAllocator::common().free_raw(standard.pointer) }.unwrap();
    let pointer = BaseAllocator::common().allocate_raw(3).unwrap();
    let common = StandardByteStorage { pointer, length: 3 };
    drop(common);
    assert!(
        include_str!("../../../../../../tools/oracle/tkernel_standard_memory_storage.expected")
            .lines()
            .any(|row| row == "common-interchange 1")
    );
}

#[test]
fn shared_header_records_exact_requested_extent_and_alignment() {
    for (size, alignment) in [(0, 1), (3, 8), (17, 64), (4096, 4096)] {
        let owned = StandardByteStorage::allocate(size, alignment, false).unwrap();
        // SAFETY: this is the original live pointer retained by the sole storage owner.
        let recorded = unsafe { super::super::raw_heap::heap_payload_size(owned.pointer) };
        assert_eq!(recorded, size);
        assert_eq!(owned.as_slice().as_ptr().addr() % alignment, 0);
        assert_eq!(owned.as_slice(), vec![0; size]);
    }
}

#[test]
fn resize_changes_the_actual_shared_allocation_extent() {
    let mut storage = StandardByteStorage::allocate(8, 16, true).unwrap();
    storage.as_mut_slice().copy_from_slice(b"abcdefgh");
    for size in [32, 3, 64, 1] {
        storage.resize(size).unwrap();
        // SAFETY: resizing replaced the owner's original raw pointer only after success.
        let recorded = unsafe { super::super::raw_heap::heap_payload_size(storage.pointer) };
        assert_eq!(recorded, size);
        assert_eq!(storage.as_slice()[0], b'a');
    }
}
