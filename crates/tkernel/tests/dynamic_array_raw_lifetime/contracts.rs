use super::super::*;
#[test]
fn raw_block_size_overflow_leaves_array_and_allocator_unchanged() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    assert!(
        array
            .storage_extension()
            .allocate_block(usize::MAX)
            .is_err()
    );
    assert_eq!(array.storage_extension().block_count(), 0);
    assert_eq!(array.size(), 0);
}
#[test]
fn invalid_publication_preserves_initialized_prefix() {
    let mut array = DynamicArray::with_increment(2);
    array.append(17_i32);
    // The request fails capacity validation before any initialization obligation arises.
    assert!(unsafe { array.storage_extension().set_used_size(3) }.is_err());
    assert_eq!(array.size(), 1);
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [17]);
}
#[test]
fn physical_capacity_is_checked_before_any_length_change() {
    let mut array = DynamicArray::<i32>::with_increment(1);
    array.append(7);
    array.clear(false);
    array.set_increment(4);
    // Native oversized retained-stride access would be invalid; reject before publication.
    assert!(unsafe { array.storage_extension().set_used_size(4) }.is_err());
    assert_eq!(array.size(), 0);
    assert!(array.value(0).is_err());
}
#[test]
fn copy_parameters_rejects_live_and_retained_blocks_without_changes() {
    let source = DynamicArray::<i32>::with_increment(8);
    let mut target = DynamicArray::with_increment(2);
    target.append(7);
    assert!(
        target
            .storage_extension()
            .copy_parameters(&source, true)
            .is_err()
    );
    assert_eq!(target.size(), 1);
    target.clear(false);
    assert!(
        target
            .storage_extension()
            .copy_parameters(&source, false)
            .is_err()
    );
    assert_eq!(target.storage_extension().available_size(), 2);
}
#[test]
fn copy_parameters_does_not_copy_values_or_rebind_kept_allocator() {
    let allocator = BaseAllocator::incremental(4096);
    let mut source = DynamicArray::with_increment(4);
    source.append(9_i32);
    let mut target = DynamicArray::with_allocator(2, Some(allocator.clone()));
    target
        .storage_extension()
        .copy_parameters(&source, false)
        .unwrap();
    assert_eq!(target.allocator(), Some(&allocator));
    assert_eq!(target.size(), 0);
    target
        .storage_extension()
        .copy_parameters(&source, true)
        .unwrap();
    assert!(target.allocator().is_none());
    assert_eq!(source.size(), 1);
}
#[test]
fn common_raw_storage_is_separate_from_registered_blocks() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    let pointer = array.storage_extension().allocate_block(1).unwrap();
    // Exactly two detached bytes are writable; no T value or registered block exists.
    unsafe {
        pointer.write(5);
        pointer.add(1).write(6);
    }
    assert_eq!(array.storage_extension().block_count(), 0);
    assert_eq!(array.size(), 0);
    unsafe {
        array.storage_extension().deallocate_block(pointer).unwrap();
    }
    unsafe {
        array
            .storage_extension()
            .deallocate_block(std::ptr::null_mut())
            .unwrap();
    }
}
#[test]
fn shrinking_without_drop_can_be_cleaned_up_explicitly() {
    RAW_LIFETIMES.with(|counts| counts.set((0, 0)));
    let mut array = DynamicArray::with_increment(2);
    let pointer = array.append(RawItem::new(7)) as *mut RawItem;
    // Publish an empty prefix before manually destroying the removed initialized value.
    unsafe {
        array.storage_extension().set_used_size(0).unwrap();
    }
    assert_eq!(RAW_LIFETIMES.with(Cell::get), (1, 0));
    unsafe {
        DynamicArrayStorageExtension::<RawItem>::destroy_at(pointer);
    }
    array.clear(true);
    assert_eq!(RAW_LIFETIMES.with(Cell::get), (1, 1));
}
