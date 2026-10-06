//! Checked constructor inputs outside the native pointer-only validation domain.
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};

#[test]
fn array1_prefix_constructor_rejects_short_storage_without_mutation() {
    let mut buffer = [3, 4];
    assert!(matches!(
        Array1::from_slice_size(&mut buffer, 3),
        Err(ArrayError::DimensionMismatch {
            expected: 3,
            actual: 2
        })
    ));
    assert_eq!(buffer, [3, 4]);
}

#[test]
fn array1_bounded_reuse_rejects_short_storage_without_mutation() {
    let mut buffer = [3, 4];
    assert!(matches!(
        Array1::from_buffer_bounds(&mut buffer, -1, 1, true),
        Err(ArrayError::DimensionMismatch {
            expected: 3,
            actual: 2
        })
    ));
    assert_eq!(buffer, [3, 4]);
}

#[test]
fn array1_bounded_slice_rejects_negative_extent_before_access() {
    let mut buffer = [3, 4];
    assert!(matches!(
        Array1::from_slice_bounds(&mut buffer, 2, 0),
        Err(ArrayError::Bounds { lower: 2, upper: 0 })
    ));
    assert_eq!(buffer, [3, 4]);
}

#[test]
fn array1_bounded_allocation_rejects_negative_extent() {
    let mut buffer = [3, 4];
    assert!(matches!(
        Array1::from_buffer_bounds(&mut buffer, 2, 0, false),
        Err(ArrayError::Bounds { lower: 2, upper: 0 })
    ));
    assert_eq!(buffer, [3, 4]);
}

#[test]
fn array1_empty_slice_keeps_the_supplied_pointer_and_requested_origin() {
    let mut buffer = [3, 4];
    let pointer = buffer.as_ptr();
    let array = Array1::from_slice_bounds(&mut buffer, -1, -2).unwrap();
    assert_eq!(array.data_ptr(), pointer);
    assert_eq!((array.lower(), array.upper()), (-1, -2));
    assert!(!array.is_deletable());
}

#[test]
fn array1_zero_size_allocation_accepts_an_empty_ignored_input() {
    let array = Array1::<i32>::from_buffer_size(&mut [], 0, false).unwrap();
    assert_eq!((array.lower(), array.upper()), (0, -1));
    assert!(array.data_ptr().is_null());
}

#[test]
fn array2_bounded_constructor_rejects_short_storage_before_mutation() {
    let mut buffer = [3, 4, 5];
    assert!(matches!(
        Array2::from_buffer_bounds(&mut buffer, -1, 0, 3, 4),
        Err(ArrayError::DimensionMismatch {
            expected: 4,
            actual: 3
        })
    ));
    assert_eq!(buffer, [3, 4, 5]);
}

#[test]
fn array2_bounded_constructor_rejects_each_negative_extent() {
    let mut buffer = [3, 4];
    assert!(matches!(
        Array2::from_buffer_bounds(&mut buffer, 2, 0, 0, 0),
        Err(ArrayError::Bounds { lower: 2, upper: 0 })
    ));
    assert!(matches!(
        Array2::from_buffer_bounds(&mut buffer, 0, 0, 3, 1),
        Err(ArrayError::Bounds { lower: 3, upper: 1 })
    ));
    assert_eq!(buffer, [3, 4]);
}

#[test]
fn bounded_array_constructors_support_nonzero_signed_origins() {
    let mut buffer = [3, 4, 5, 6];
    let array1 = Array1::from_slice_bounds(&mut buffer, i32::MIN, i32::MIN + 3).unwrap();
    assert_eq!(array1[i32::MIN + 3], 6);
    drop(array1);
    let array2 = Array2::from_buffer_bounds(&mut buffer, -5, -4, -3, -2).unwrap();
    assert_eq!(array2[(-4, -2)], 6);
    assert_eq!(
        (array2.array1().lower(), array2.array1().upper()),
        (-13, -10)
    );
}
