//! Overflowing unsigned increments preserve metadata but cannot allocate typed zero-stride blocks.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, OccAllocator};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[test]
fn public_overflow_constructors_allow_empty_queries_and_clear() {
    let allocator = BaseAllocator::fresh_common();
    let typed = OccAllocator::new(allocator.clone());
    let arrays = [
        DynamicArray::<i32>::with_increment(usize::MAX),
        DynamicArray::with_allocator(usize::MAX, Some(allocator)),
        DynamicArray::with_occ_allocator(usize::MAX, &typed),
    ];
    for mut array in arrays {
        assert_eq!(
            (array.size(), array.length(), array.lower(), array.upper()),
            (0, 0, 0, -1)
        );
        assert!(array.is_empty());
        assert_eq!(array.iter().count(), 0);
        array.erase_last();
        array.clear(true);
        array.set_increment(0);
        array.append(9);
        assert_eq!(array[0], 9);
    }
}
#[test]
fn zero_stride_checked_access_and_protected_allocation_reject_without_changes() {
    let mut array = DynamicArray::<i32>::with_increment(usize::MAX);
    assert!(array.value(0).is_err());
    assert!(array.change_value(0).is_err());
    let mut storage = array.storage_extension();
    assert!(
        storage
            .ensure_storage_for_index(0, size_of::<i32>())
            .is_err()
    );
    assert!(storage.ensure_block_count(1, size_of::<i32>()).is_err());
    assert!(storage.expand_block(size_of::<i32>()).is_err());
    storage.ensure_block_count(0, size_of::<i32>()).unwrap();
    assert_eq!(storage.block_count(), 0);
    assert_eq!(storage.available_size(), 0);
}
#[test]
fn zero_stride_factories_and_indexing_reject_before_callbacks() {
    let mut array = DynamicArray::<i32>::with_increment(usize::MAX);
    let mut calls = 0;
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.emplace_append(|| {
                calls += 1;
                9
            });
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array
                .emplace_value(0, || {
                    calls += 1;
                    9
                })
                .ok();
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array[0] = 9;
        }))
        .is_err()
    );
    assert_eq!(calls, 0);
    assert_eq!(array.size(), 0);
    assert_eq!(array.storage_extension().block_count(), 0);
    array.set_increment(2);
    array.append(4);
    assert_eq!(array[0], 4);
}
#[test]
fn copying_and_moving_zero_metadata_does_not_normalize_it_to_one() {
    let mut source = DynamicArray::<i32>::with_increment(usize::MAX);
    let mut copy = source.clone();
    let mut moved = DynamicArray::from_moved(&mut source);
    for array in [&mut source, &mut copy, &mut moved] {
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                array.append(3);
            }))
            .is_err()
        );
        assert_eq!(array.size(), 0);
        array.set_increment_i32(-1);
        array.append(3);
        assert_eq!(array[0], 3);
    }
}
#[test]
fn nonempty_increment_change_ignores_overflow_and_preserves_storage() {
    let mut array = DynamicArray::with_increment(2);
    let before = array.append(7) as *const i32;
    array.set_increment(usize::MAX);
    array.append(8);
    assert_eq!(before, &array[0]);
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [7, 8]);
}
