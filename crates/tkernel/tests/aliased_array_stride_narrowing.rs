//! Native checkedStride narrows only after testing the original size_t input.
use rustycad_tkernel::ncollection::AliasedArray;

#[test]
fn checked_size_preserves_positive_signed_domain() {
    for count in [i32::MIN, -1, 0] {
        let error = AliasedArray::checked_size(count).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_RangeError");
    }
    assert_eq!(AliasedArray::checked_size(1).unwrap(), 1);
    assert_eq!(
        AliasedArray::checked_size(i32::MAX).unwrap(),
        i32::MAX as usize
    );
}

#[test]
fn checked_stride_rejects_only_original_zero() {
    assert!(AliasedArray::checked_stride(0).is_err());
    assert_eq!(AliasedArray::checked_stride(1).unwrap(), 1);
    assert_eq!(
        AliasedArray::checked_stride(u32::MAX as usize).unwrap(),
        u32::MAX
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn empty_copy_preserves_wrapped_zero_stride() {
    let stride: usize = (u32::MAX as usize) + 1;
    assert_eq!(AliasedArray::checked_stride(stride).unwrap(), 0);
    let empty = AliasedArray::empty(stride).unwrap();
    let copy = empty.clone();
    for array in [&empty, &copy] {
        assert_eq!(array.stride(), 0);
        assert_eq!(array.size_bytes(), 0);
        assert!(array.is_empty());
        assert!(!array.is_allocated());
    }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn allocated_array_uses_narrowed_stride_for_all_access() {
    let stride: usize = (u32::MAX as usize) + 5;
    let mut array = AliasedArray::new(stride, 2).unwrap();
    assert_eq!(array.stride(), 4);
    assert_eq!(array.size_bytes(), 8);
    array.init(17_u32).unwrap();
    array.resize(3, true).unwrap();
    assert_eq!(*array.value::<u32>(1).unwrap(), 17);
    assert_eq!(array.clone().size_bytes(), 12);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn signed_constructor_validates_size_before_narrowing_stride() {
    let stride: usize = (u32::MAX as usize) + 5;
    assert!(AliasedArray::new_signed(stride, -1).is_err());
    let array = AliasedArray::new_signed(stride, 2).unwrap();
    assert_eq!(array.stride(), 4);
    assert_eq!(array.length(), 2);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn allocated_copy_preserves_wrapped_zero_stride() {
    let stride: usize = (u32::MAX as usize) + 1;
    let array = AliasedArray::new(stride, 2).unwrap();
    let copy = array.clone();
    for value in [&array, &copy] {
        assert_eq!(value.stride(), 0);
        assert_eq!(value.len(), 2);
        assert_eq!(value.size_bytes(), 0);
        assert!(value.is_allocated());
    }
}
