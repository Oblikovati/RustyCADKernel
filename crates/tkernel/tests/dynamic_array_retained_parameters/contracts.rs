use super::super::*;
fn live_array(allocator: BaseAllocator) -> (DynamicArray<ParameterItem>, Rc<ParameterCounts>) {
    let counts = Rc::new(ParameterCounts::default());
    let mut array = DynamicArray::with_allocator(2, Some(allocator));
    array.append(ParameterItem::new(7, &counts));
    (array, counts)
}
#[test]
fn incompatible_arena_rejection_preserves_live_values_and_owner() {
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let (mut target, counts) = live_array(allocator.clone());
    let source = DynamicArray::with_allocator(2, Some(BaseAllocator::accumulating(4096).unwrap()));
    let error = unsafe {
        target
            .storage_extension()
            .copy_parameters_retained(&source, true)
    }
    .unwrap_err();
    assert!(error.message().contains("compatible typed allocator"));
    assert_eq!(target.allocator(), Some(&allocator));
    assert_eq!(retained_values(&target), [7]);
    assert_eq!(counts.destroyed.get(), 0);
    drop(target);
    assert_eq!(counts.destroyed.get(), 1);
}
fn mixed_capacity_array() -> (DynamicArray<ParameterItem>, Rc<ParameterCounts>) {
    let counts = Rc::new(ParameterCounts::default());
    let mut target = DynamicArray::with_increment(4);
    target
        .storage_extension()
        .ensure_block_count(1, size_of::<ParameterItem>())
        .unwrap();
    target.set_increment(2);
    for value in 1..=3 {
        target.append(ParameterItem::new(value, &counts));
    }
    (target, counts)
}
#[test]
fn every_physical_block_is_validated_before_forgetting_any_prefix() {
    let (mut target, counts) = mixed_capacity_array();
    let source = DynamicArray::with_increment(4);
    let error = unsafe {
        target
            .storage_extension()
            .copy_parameters_retained(&source, false)
    }
    .unwrap_err();
    assert!(
        error
            .message()
            .contains("block 1, capacity 2, requested stride 4")
    );
    assert_eq!(retained_values(&target), [1, 2, 3]);
    assert_eq!(counts.destroyed.get(), 0);
    drop(target);
    assert_eq!(counts.destroyed.get(), 3);
}
#[test]
fn same_aligned_allocator_preserves_alignment_and_republished_value() {
    let allocator = BaseAllocator::aligned(64);
    let mut target = DynamicArray::with_allocator(2, Some(allocator.clone()));
    target.append(42_i32);
    let pointer = target.storage_extension().change_block_at(0).unwrap();
    let source = DynamicArray::with_allocator(2, Some(allocator));
    unsafe {
        target
            .storage_extension()
            .copy_parameters_retained(&source, true)
            .unwrap()
    };
    assert_eq!(pointer as usize % 64, 0);
    assert_eq!(unsafe { *pointer }, 42);
    unsafe { target.storage_extension().set_used_size(1).unwrap() };
    assert_eq!(target.value(0).unwrap(), &42);
}
#[test]
fn different_aligned_layout_is_rejected_without_rebinding() {
    let owner = BaseAllocator::aligned(64);
    let mut target = DynamicArray::with_allocator(2, Some(owner.clone()));
    target.append(9_i32);
    let source = DynamicArray::with_allocator(2, Some(BaseAllocator::aligned(128)));
    assert!(
        unsafe {
            target
                .storage_extension()
                .copy_parameters_retained(&source, true)
        }
        .is_err()
    );
    assert_eq!(target.allocator(), Some(&owner));
    assert_eq!(target.value(0).unwrap(), &9);
}
#[test]
fn retained_zst_is_outside_the_native_physical_layout_domain() {
    let mut target = DynamicArray::with_increment(2);
    target.append(());
    let source = DynamicArray::with_increment(2);
    let error = unsafe {
        target
            .storage_extension()
            .copy_parameters_retained(&source, false)
    }
    .unwrap_err();
    assert!(error.message().contains("item size 0"));
    assert_eq!(target.size(), 1);
}
#[test]
fn unallocated_destination_can_adopt_a_different_arena() {
    let source = DynamicArray::<i32>::with_allocator(8, Some(BaseAllocator::incremental(4096)));
    let mut target =
        DynamicArray::with_allocator(2, Some(BaseAllocator::accumulating(4096).unwrap()));
    unsafe {
        target
            .storage_extension()
            .copy_parameters_retained(&source, true)
            .unwrap()
    };
    assert_eq!(target.allocator(), source.allocator());
    target.append(5);
    assert_eq!(target.storage_extension().available_size(), 8);
    assert_eq!(target.value(0).unwrap(), &5);
}
