use super::super::*;
type RawStorage<'a> = DynamicArrayStorageExtension<'a, RawItem>;
fn snapshot_row(
    output: &mut String,
    name: &str,
    array: &mut DynamicArray<RawItem>,
    allocator: &BaseAllocator,
    counts: &mut RawAllocationObservations,
) {
    output_row(output, name, &raw_snapshot(array, allocator, counts));
}
fn raw_allocation(array: &mut DynamicArray<RawItem>) -> *mut RawItem {
    array
        .storage_extension()
        .allocate_block(size_of::<RawItem>())
        .unwrap()
        .cast()
}
unsafe fn free_raw_allocation(array: &mut DynamicArray<RawItem>, pointer: *mut RawItem) {
    // Caller passes this array allocator's detached raw pointer, with no remaining live value.
    unsafe {
        array
            .storage_extension()
            .deallocate_block(pointer.cast())
            .unwrap();
    }
}
fn detached_value_rows(
    output: &mut String,
    array: &mut DynamicArray<RawItem>,
    allocator: &BaseAllocator,
    counts: &mut RawAllocationObservations,
    pointer: *mut RawItem,
) {
    // This is raw-policy storage, aligned for RawItem and exclusively owned until raw free.
    unsafe {
        RawStorage::construct_at(pointer, || RawItem::new(41));
    }
    let value = unsafe { (*pointer).value as usize };
    output_row(output, "detached_value", &[value]);
    snapshot_row(output, "detached_construct", array, allocator, counts);
}
fn detached_rows(
    output: &mut String,
    array: &mut DynamicArray<RawItem>,
    allocator: &BaseAllocator,
    counts: &mut RawAllocationObservations,
) {
    let pointer = raw_allocation(array);
    snapshot_row(output, "detached_alloc", array, allocator, counts);
    detached_value_rows(output, array, allocator, counts, pointer);
    unsafe {
        RawStorage::destroy_at(pointer);
    }
    snapshot_row(output, "detached_destroy", array, allocator, counts);
    unsafe {
        free_raw_allocation(array, pointer);
    }
    snapshot_row(output, "detached_free", array, allocator, counts);
}
fn construct_three(array: &mut DynamicArray<RawItem>) -> (*mut RawItem, *mut RawItem) {
    let mut storage = array.storage_extension();
    storage.ensure_block_count(2, size_of::<RawItem>()).unwrap();
    let first = storage.change_block_at(0).unwrap();
    let last = storage.change_block_at(1).unwrap();
    // Every raw slot is inside allocated capacity and not yet owned by Vec lengths.
    unsafe {
        RawStorage::construct_at(first, || RawItem::new(10));
        RawStorage::construct_at(first.add(1), || RawItem::new(20));
        RawStorage::construct_at(last, || RawItem::new(30));
    }
    (first, last)
}
fn publication_rows(
    output: &mut String,
    array: &mut DynamicArray<RawItem>,
    allocator: &BaseAllocator,
    counts: &mut RawAllocationObservations,
) {
    let (first, last) = construct_three(array);
    snapshot_row(output, "constructed_unpublished", array, allocator, counts);
    // All three prefix slots were constructed above and transfer unique ownership to the Vecs.
    unsafe {
        array.storage_extension().set_used_size(3).unwrap();
    }
    output_row(output, "published_values", &raw_values(array));
    snapshot_row(output, "published_three", array, allocator, counts);
    shrink_restore_rows(output, array, allocator, counts);
    destroy_removed(first, last, array);
    snapshot_row(output, "destroyed_removed", array, allocator, counts);
    array.clear(true);
    snapshot_row(output, "cleared_published", array, allocator, counts);
}
fn shrink_restore_rows(
    output: &mut String,
    array: &mut DynamicArray<RawItem>,
    allocator: &BaseAllocator,
    counts: &mut RawAllocationObservations,
) {
    // Shrink forgets automatic ownership; no bytes are moved or destroyed before restoration.
    unsafe {
        array.storage_extension().set_used_size(1).unwrap();
    }
    snapshot_row(output, "shrunk_one", array, allocator, counts);
    unsafe {
        array.storage_extension().set_used_size(3).unwrap();
    }
    snapshot_row(output, "restored_three", array, allocator, counts);
    output_row(output, "restored_values", &raw_values(array));
}
fn destroy_removed(first: *mut RawItem, last: *mut RawItem, array: &mut DynamicArray<RawItem>) {
    // Remove both tail slots from automatic ownership before manually ending their lifetimes.
    unsafe {
        array.storage_extension().set_used_size(1).unwrap();
        RawStorage::destroy_at(first.add(1));
        RawStorage::destroy_at(last);
    }
}
fn throwing_rows(
    output: &mut String,
    array: &mut DynamicArray<RawItem>,
    allocator: &BaseAllocator,
    counts: &mut RawAllocationObservations,
) {
    let pointer = raw_allocation(array);
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        RawStorage::construct_at(pointer, || RawItem::new(99));
    }));
    assert!(result.is_err());
    snapshot_row(output, "failed_raw_constructor", array, allocator, counts);
    unsafe {
        retry_failed_constructor(array, pointer);
    }
    snapshot_row(output, "raw_retry_free", array, allocator, counts);
}
unsafe fn retry_failed_constructor(array: &mut DynamicArray<RawItem>, pointer: *mut RawItem) {
    // The failed constructor never created a live value; reconstruct, destroy, then raw free.
    unsafe {
        RawStorage::construct_at(pointer, || RawItem::new(42));
        RawStorage::destroy_at(pointer);
        free_raw_allocation(array, pointer);
    }
}
fn parameter_case(source: &DynamicArray<RawItem>, own: &BaseAllocator, adopt: bool) -> [usize; 3] {
    let mut target = DynamicArray::with_allocator(2, Some(own.clone()));
    target
        .storage_extension()
        .copy_parameters(source, adopt)
        .unwrap();
    let allocator = target.allocator().unwrap().clone();
    let before = allocator_state(&allocator).1;
    let pointer = raw_allocation(&mut target);
    let bytes = before - allocator_state(&allocator).1;
    let expected = if adopt { source.allocator() } else { Some(own) };
    let same = usize::from(target.allocator() == expected);
    // The raw block is still paired with its creating allocator and contains no initialized T.
    unsafe {
        free_raw_allocation(&mut target, pointer);
    }
    [same, target.size(), bytes]
}
fn parameter_rows(output: &mut String) {
    let own = BaseAllocator::accumulating(4096).unwrap();
    let other = BaseAllocator::accumulating(4096).unwrap();
    let source = DynamicArray::<RawItem>::with_allocator(4, Some(other));
    let keep = parameter_case(&source, &own, false);
    let adopt = parameter_case(&source, &own, true);
    let values = [keep[0], adopt[0], keep[1], adopt[1], keep[2], adopt[2]];
    output_row(output, "empty_copy_parameters", &values);
}
#[test]
fn native_raw_lifetime_rows_match_runtime_observations() {
    RAW_LIFETIMES.with(|counts| counts.set((0, 0)));
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = DynamicArray::with_allocator(2, Some(allocator.clone()));
    let mut counts = RawAllocationObservations::new(&allocator);
    let mut actual = String::new();
    detached_rows(&mut actual, &mut array, &allocator, &mut counts);
    publication_rows(&mut actual, &mut array, &allocator, &mut counts);
    throwing_rows(&mut actual, &mut array, &allocator, &mut counts);
    parameter_rows(&mut actual);
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkernel_dynamic_raw_lifetime.expected")
    );
}
