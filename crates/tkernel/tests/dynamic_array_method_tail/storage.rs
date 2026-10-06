use super::super::{BaseAllocator, DynamicArray, OccAllocator};
#[test]
#[allow(deprecated)]
fn deprecated_vector_alias_preserves_dynamic_storage() {
    let mut vector = rustycad_tkernel::ncollection::Vector::with_increment_i32(-1);
    vector.append(4);
    assert_eq!(vector.size(), 1);
    assert_eq!(*vector.last().unwrap(), 4);
}
#[test]
fn signed_constructor_overloads_preserve_resources() {
    let allocator = BaseAllocator::incremental(4096);
    let mut direct = DynamicArray::with_allocator_i32(-4, Some(allocator.clone()));
    direct.append(1_i32);
    assert_eq!(direct.allocator(), Some(&allocator));
    let typed = OccAllocator::new(allocator.clone());
    let mut through_typed = DynamicArray::with_occ_allocator_i32(0, &typed);
    through_typed.append(2_i32);
    assert_eq!(through_typed.allocator(), Some(&allocator));
    assert_eq!(through_typed.size(), 1);
}
#[test]
fn same_increment_preserves_retained_element_block() {
    let allocator = BaseAllocator::incremental(4096);
    let mut array = DynamicArray::with_allocator(4, Some(allocator));
    let original = array.append(1_i32) as *const i32;
    array.clear(false);
    array.set_increment(4);
    assert_eq!(array.append(2) as *const i32, original);
}
#[test]
fn signed_increment_clamps_and_retains_empty_storage() {
    let allocator = BaseAllocator::incremental(4096);
    let mut array = DynamicArray::with_allocator_i32(-3, Some(allocator));
    let original = array.append(1_i32) as *const i32;
    array.clear(false);
    array.set_increment_i32(-8);
    assert_eq!(array.append(2) as *const i32, original);
    array.set_increment_i32(8);
    assert_eq!(*array.first().unwrap(), 2);
}
#[test]
fn smaller_increment_reuses_native_valid_storage() {
    let allocator = BaseAllocator::incremental(4096);
    let mut array = DynamicArray::with_allocator(4, Some(allocator));
    let original = array.append(1_i32) as *const i32;
    array.clear(false);
    array.set_increment(2);
    assert_eq!(array.append(2) as *const i32, original);
    array.append(3);
    array.append(4);
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [2, 3, 4]);
}
#[test]
fn native_self_assignment_adapters_keep_identity_and_storage() {
    let mut array = DynamicArray::with_increment(2);
    let original = array.append(7_i32) as *const i32;
    let identity = &raw mut array;
    assert!(std::ptr::eq(array.assign_self(true), identity));
    assert!(std::ptr::eq(array.assign_self(false), identity));
    assert!(std::ptr::eq(array.assign_operator_self(), identity));
    assert!(std::ptr::eq(array.move_self(), identity));
    assert_eq!(array.first().unwrap() as *const i32, original);
    assert_eq!(array.size(), 1);
}
