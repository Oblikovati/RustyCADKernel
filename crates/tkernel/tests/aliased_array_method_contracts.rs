//! Method-level regression coverage for NCollection_AliasedArray's pointer and legacy overloads.
use rustycad_tkernel::{ncollection::AliasedArray, standard::Failure};

fn assert_aliased_failure<T>(result: Result<T, Failure>, expected: &str) {
    let failure: Failure = result.err().expect("expected an AliasedArray exception");
    assert_eq!(failure.exception_type(), expected);
}

#[test]
fn empty_typed_pointers_preserve_null_and_check_stride() {
    let mut empty: AliasedArray<'_> = AliasedArray::empty(4).unwrap();
    assert!(empty.data_ptr::<i32>().unwrap().is_null());
    assert!(empty.data_mut_ptr::<i32>().unwrap().is_null());
    assert_aliased_failure(empty.data_ptr::<i64>(), "Standard_TypeMismatch");
    assert_aliased_failure(empty.data_mut_ptr::<i64>(), "Standard_TypeMismatch");
    assert!(empty.byte_value_ptr(0).is_err());
    assert!(empty.change_byte_value_ptr(0).is_err());
}

#[test]
fn pointer_offsets_match_external_element_stride() {
    let mut values: [i32; 3] = [10, 20, 30];
    let beginning: *mut i32 = values.as_mut_ptr();
    let mut view: AliasedArray<'_> = AliasedArray::from_slice(&mut values).unwrap();
    assert_eq!(view.data_ptr::<i32>().unwrap(), beginning.cast_const());
    assert_eq!(view.data_mut_ptr::<i32>().unwrap(), beginning);
    let second: *const u8 = beginning.wrapping_add(1).cast_const().cast();
    let third: *mut u8 = beginning.wrapping_add(2).cast();
    assert_eq!(view.byte_value_ptr(1).unwrap(), second);
    assert_eq!(view.change_byte_value_ptr(2).unwrap(), third);
    assert_eq!(view.byte_value_ptr_signed(1).unwrap(), second);
    assert_eq!(view.change_byte_value_ptr_signed(2).unwrap(), third);
    assert!(view.byte_value_ptr_signed(-1).is_err());
    assert!(view.change_byte_value_ptr_signed(-1).is_err());
}

#[test]
fn signed_lengths_validate_before_allocating_or_borrowing() {
    assert!(AliasedArray::new_signed(4, -1).is_err());
    assert!(AliasedArray::new_signed(4, 0).is_err());
    let mut values: [i32; 3] = [10, 20, 30];
    assert!(AliasedArray::from_slice_signed(&mut values, -1).is_err());
    assert!(AliasedArray::from_slice_signed(&mut values, 4).is_err());
    let mut view: AliasedArray<'_> = AliasedArray::from_slice_signed(&mut values, 2).unwrap();
    assert_eq!(view.length(), 2);
    view.resize_signed(3, true).unwrap();
    assert_eq!(*view.value::<i32>(1).unwrap(), 20);
    assert!(view.is_allocated());
    assert!(view.resize_signed(0, false).is_err());
    assert_eq!(view.length(), 3);
}

#[test]
fn signed_element_access_checks_negative_before_type() {
    let mut array: AliasedArray<'_> = AliasedArray::new_signed(4, 2).unwrap();
    *array.change_value_signed::<i32>(0).unwrap() = 17;
    assert_eq!(*array.value_signed::<i32>(0).unwrap(), 17);
    assert_aliased_failure(array.value_signed::<i64>(-1), "Standard_OutOfRange");
    assert_aliased_failure(array.change_value_signed::<i64>(-1), "Standard_OutOfRange");
    assert_aliased_failure(array.value_signed::<i64>(2), "Standard_TypeMismatch");
    assert!(array.change_value_signed::<i32>(2).is_err());
}

#[test]
fn signed_prefix_access_checks_negative_before_type() {
    let mut array: AliasedArray<'_> = AliasedArray::new_signed(4, 2).unwrap();
    *array.change_value2_signed::<u16>(1).unwrap() = 23;
    assert_eq!(*array.value2_signed::<u16>(1).unwrap(), 23);
    assert_aliased_failure(array.value2_signed::<u64>(-1), "Standard_OutOfRange");
    assert_aliased_failure(array.change_value2_signed::<u64>(-1), "Standard_OutOfRange");
    assert_aliased_failure(array.value2_signed::<u64>(2), "Standard_TypeMismatch");
    assert!(array.change_value2_signed::<u16>(2).is_err());
}

#[test]
fn output_overloads_copy_without_mutating_source() {
    let mut array: AliasedArray<'_> = AliasedArray::new_signed(4, 2).unwrap();
    array.init(17_i32).unwrap();
    let mut whole: i32 = 0;
    let mut prefix: u8 = 0;
    array.value_into(0, &mut whole).unwrap();
    assert_eq!(whole, 17);
    array.value_into_signed(1, &mut whole).unwrap();
    assert_eq!(whole, 17);
    array.value2_into(0, &mut prefix).unwrap();
    assert_eq!(prefix, 17_i32.to_ne_bytes()[0]);
    array.value2_into_signed(1, &mut prefix).unwrap();
    assert_eq!(prefix, 17_i32.to_ne_bytes()[0]);
    assert_eq!(*array.value::<i32>(0).unwrap(), 17);
}

#[test]
fn output_overload_failure_preserves_destination() {
    let array: AliasedArray<'_> = AliasedArray::new_signed(4, 2).unwrap();
    let mut output: u64 = 99;
    assert_aliased_failure(
        array.value_into_signed(-1, &mut output),
        "Standard_OutOfRange",
    );
    assert_aliased_failure(
        array.value2_into_signed(-1, &mut output),
        "Standard_OutOfRange",
    );
    assert_aliased_failure(array.value_into(2, &mut output), "Standard_TypeMismatch");
    assert_aliased_failure(array.value2_into(2, &mut output), "Standard_TypeMismatch");
    assert_eq!(output, 99);
}

#[test]
fn assignment_returns_destination_and_preserves_borrowed_storage() {
    let mut source: AliasedArray<'_> = AliasedArray::new_signed(4, 2).unwrap();
    source.init(42_i32).unwrap();
    let mut values: [i32; 2] = [0, 0];
    let beginning: *mut i32 = values.as_mut_ptr();
    let mut view: AliasedArray<'_> = AliasedArray::from_slice(&mut values).unwrap();
    let identity: *mut AliasedArray<'_> = &mut view;
    assert_eq!(view.assign(&source).unwrap() as *mut _, identity);
    assert_eq!(view.assign_self() as *mut _, identity);
    assert_eq!(view.data_ptr::<i32>().unwrap(), beginning.cast_const());
    assert!(!view.is_deletable());
    drop(view);
    assert_eq!(values, [42, 42]);
}

#[test]
fn moving_returns_destination_but_safely_empties_the_source_alias() {
    let mut source: AliasedArray<'_> = AliasedArray::new_signed(4, 2).unwrap();
    source.init(23_i32).unwrap();
    let mut destination: AliasedArray<'_> = AliasedArray::new_signed(8, 3).unwrap();
    let beginning: *const i32 = source.data_ptr::<i32>().unwrap();
    let identity: *mut AliasedArray<'_> = &mut destination;
    assert_eq!(destination.move_from(&mut source) as *mut _, identity);
    assert_eq!(destination.data_ptr::<i32>().unwrap(), beginning);
    assert_eq!(*destination.value::<i32>(1).unwrap(), 23);
    assert!(destination.is_deletable());
    // OCCT leaves a dangling-capable non-owning source alias; this safe API cannot do so.
    assert!(source.is_empty());
    assert!(!source.is_deletable());
    assert!(source.data_ptr::<i32>().unwrap().is_null());
}
