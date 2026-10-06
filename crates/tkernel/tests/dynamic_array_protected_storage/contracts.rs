use super::super::*;
#[test]
fn pointer_access_checks_blocks_without_referencing_uninitialized_slots() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    let mut storage = array.storage_extension();
    assert!(
        storage
            .block_address(0)
            .unwrap_err()
            .message()
            .contains("block index 0")
    );
    assert!(storage.change_block_address(0).is_err());
    storage.ensure_block_count(1, size_of::<i32>()).unwrap();
    assert!(!storage.block_at(0).unwrap().is_null());
    assert!(!storage.change_block_at(0).unwrap().is_null());
    assert!(storage.at(0).is_err());
    assert!(storage.change_at(0).is_err());
    assert!(storage.block_at(1).is_err());
    assert!(storage.change_block_at(1).is_err());
}
#[test]
fn typed_reservations_reject_item_sizes_before_allocating() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    let mut storage = array.storage_extension();
    assert!(storage.ensure_block_count(3, 1).is_err());
    assert!(storage.ensure_storage_for_index(8, 8).is_err());
    assert!(storage.expand_block(0).is_err());
    assert_eq!(storage.block_count(), 0);
    assert_eq!(storage.available_size(), 0);
}
#[test]
fn storage_size_guards_reject_overflow_without_allocation() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    let mut storage = array.storage_extension();
    assert!(storage.ensure_block_count(usize::MAX, 4).is_err());
    assert!(storage.ensure_storage_for_index(usize::MAX, 4).is_err());
    assert_eq!(storage.block_count(), 0);
    let mut unit_stride = DynamicArray::<i32>::with_increment(1);
    assert!(
        unit_stride
            .storage_extension()
            .ensure_storage_for_index(usize::MAX, 4)
            .is_err()
    );
}
#[test]
fn zero_sized_rust_elements_are_outside_the_native_typed_block_domain() {
    let mut array = DynamicArray::<()>::default();
    assert!(array.storage_extension().ensure_block_count(1, 0).is_err());
    assert!(array.storage_extension().expand_block(0).is_err());
    assert_eq!(array.storage_extension().block_count(), 0);
}
#[test]
fn clear_blocks_forgets_values_but_releases_real_allocations() {
    ELEMENT_LIFETIMES.with(|counts| counts.set((0, 0)));
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = DynamicArray::with_allocator(2, Some(allocator.clone()));
    for value in 0..5 {
        array.append(EngineItem::new(value));
    }
    array.storage_extension().clear_blocks();
    assert_eq!(array.size(), 0);
    assert_eq!(array.storage_extension().block_count(), 0);
    assert_eq!(ELEMENT_LIFETIMES.with(Cell::get), (5, 0));
    assert_eq!(live_allocations(&allocator), 0);
    array.append(EngineItem::new(7));
    array.clear(true);
    assert_eq!(ELEMENT_LIFETIMES.with(Cell::get), (6, 1));
}
#[test]
fn clear_empty_blocks_is_idempotent_and_retains_allocator_identity() {
    let allocator = BaseAllocator::incremental(4096);
    let mut array = DynamicArray::<i32>::with_allocator(2, Some(allocator.clone()));
    array.storage_extension().clear_blocks();
    array.storage_extension().ensure_block_count(3, 4).unwrap();
    array.storage_extension().clear_blocks();
    array.storage_extension().clear_blocks();
    assert_eq!(array.allocator(), Some(&allocator));
    assert_eq!(*array.append(9), 9);
}
#[test]
fn empty_reservation_does_not_require_default_or_clone() {
    let mut array = DynamicArray::<EngineItem>::with_increment(4);
    array
        .storage_extension()
        .ensure_storage_for_index(7, size_of::<EngineItem>())
        .unwrap();
    assert_eq!(array.size(), 0);
    array.emplace_append(|| EngineItem::new(8));
    assert_eq!(array.storage_extension().at(0).unwrap().value, 8);
    assert_eq!(array.storage_extension().available_size(), 8);
}
