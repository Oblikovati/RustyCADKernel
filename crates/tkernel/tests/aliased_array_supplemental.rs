//! Checked representation, copy ownership and invalid-shape boundaries.
use rustycad_tkernel::ncollection::AliasedArray;
#[test]
fn runtime_reinterpretation_partial_views_and_same_size_resize() {
    let mut array = AliasedArray::new(8, 2).unwrap();
    array.init(0_u64).unwrap();
    *array.change_value::<u64>(0).unwrap() = 0x0102_0304_0506_0708;
    assert_eq!(
        *array.value2::<u8>(0).unwrap(),
        0x0102_0304_0506_0708_u64.to_ne_bytes()[0]
    );
    *array.change_value2::<u16>(1).unwrap() = 0x1234;
    assert_eq!(*array.value2::<u16>(1).unwrap(), 0x1234);
    let pointer: *const u64 = array.value(0).unwrap();
    array.resize(2, false).unwrap();
    assert_eq!(pointer, array.value::<u64>(0).unwrap() as *const u64);
    assert_eq!(*array.value::<u64>(0).unwrap(), 0x0102_0304_0506_0708);
    array.resize(3, true).unwrap();
    assert_eq!(*array.value2::<u16>(1).unwrap(), 0x1234);
    assert_eq!(array.size_bytes(), 24);
}
#[test]
fn borrowed_copy_becomes_independent_and_assignment_keeps_owner() {
    let mut external: [i32; 2] = [10, 20];
    let mut borrowed = AliasedArray::from_slice(&mut external).unwrap();
    let mut copy = borrowed.clone();
    assert!(copy.is_deletable());
    *copy.change_value::<i32>(0).unwrap() = 99;
    assert_eq!(*borrowed.value::<i32>(0).unwrap(), 10);
    borrowed.assign(&copy).unwrap();
    assert!(!borrowed.is_deletable());
    borrowed.resize(3, true).unwrap();
    assert!(borrowed.is_deletable());
    assert_eq!(*borrowed.value::<i32>(1).unwrap(), 20);
    drop(borrowed);
    assert_eq!(external, [99, 20]);
    let empty = AliasedArray::empty(4).unwrap().clone();
    assert!(empty.is_empty());
    assert!(!empty.is_deletable());
    assert!(empty.typed_data::<i32>().unwrap().is_empty());
}
#[test]
fn invalid_dimensions_types_bounds_and_alignment_are_checked() {
    assert!(AliasedArray::empty(0).is_err());
    assert!(AliasedArray::new(4, 0).is_err());
    assert!(AliasedArray::from_slice::<i32>(&mut []).is_err());
    let mut array = AliasedArray::new(4, 2).unwrap();
    assert!(array.resize(0, true).is_err());
    assert!(array.resize(usize::MAX, false).is_err());
    assert!(array.assign(&AliasedArray::new(8, 2).unwrap()).is_err());
    assert_eq!(
        array.value::<u64>(0).unwrap_err().exception_type(),
        "Standard_TypeMismatch"
    );
    assert!(array.change_data::<u64>().is_err());
    assert!(array.value::<i32>(2).is_err());
    assert!(array.change_value::<i32>(2).is_err());
    let mut aligned: [u128; 1] = [0];
    let bytes: &mut [u8] = bytemuck::cast_slice_mut(&mut aligned);
    let unaligned = AliasedArray::from_slice(&mut bytes[1..]).unwrap();
    assert!(unaligned.value::<u16>(0).is_err());
    let mut odd_stride = AliasedArray::new(3, 2).unwrap();
    assert!(odd_stride.change_value2::<u16>(1).is_err());
}
