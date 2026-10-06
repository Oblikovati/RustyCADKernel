#![allow(unsafe_code)]
//! Validity and Rust-specific safety boundaries for raw array compatibility.
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};

#[derive(Copy, Debug, PartialEq)]
struct RawCopyWithoutClone(i32);

// This deliberate probe must fail if the explicit trivial policy invokes Clone.
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for RawCopyWithoutClone {
    fn clone(&self) -> Self {
        panic!(
            "clone requested for {:?}; expected the explicit trivial copy policy",
            self
        );
    }
}

#[test]
fn raw_assignment_allocates_without_invoking_clone() {
    let source = [RawCopyWithoutClone(3), RawCopyWithoutClone(4)];
    let mut destination = Array1::default();
    unsafe {
        destination.assign_trivial_from_raw_parts(source.as_ptr(), 2, 7);
    }
    assert_eq!(destination.as_slice(), source);
    assert_eq!(destination.lower(), 7);
}

#[test]
fn raw_assignment_reuses_borrowed_storage_without_invoking_clone() {
    let source = [RawCopyWithoutClone(3), RawCopyWithoutClone(4)];
    let mut buffer = [RawCopyWithoutClone(0); 2];
    let mut destination = Array1::from_slice(&mut buffer);
    unsafe {
        destination.assign_trivial_from_raw_parts(source.as_ptr(), 2, 7);
    }
    assert_eq!(destination.as_slice(), source);
    assert!(!destination.is_deletable());
}

#[test]
fn raw_empty_assignment_releases_owned_storage_and_keeps_source_lower() {
    let mut destination = Array1::<i32>::with_size(2);
    unsafe {
        destination.assign_trivial_from_raw_parts(std::ptr::null(), 0, 7);
    }
    assert_eq!((destination.lower(), destination.upper()), (7, 6));
    assert!(destination.data_ptr().is_null());
    assert!(!destination.is_deletable());
}

#[test]
fn raw_empty_assignment_detaches_borrowed_storage_without_writing() {
    let mut buffer = [1, 2];
    let mut destination = Array1::from_slice(&mut buffer);
    unsafe {
        destination.assign_trivial_from_raw_parts(std::ptr::null(), 0, 7);
    }
    assert!(destination.data_ptr().is_null());
    drop(destination);
    assert_eq!(buffer, [1, 2]);
}

#[test]
fn raw_rectangle_copy_returns_receiver_and_preserves_flat_lower() {
    let mut destination = Array2::<i32>::with_bounds(-1, 0, 7, 8).unwrap();
    let source = [1, 2, 3, 4];
    let receiver = std::ptr::from_mut(&mut destination);
    let result = unsafe {
        destination
            .copy_values_trivial_from_raw_parts(source.as_ptr(), 2, 2)
            .unwrap()
    };
    assert!(std::ptr::eq(receiver, result));
    assert_eq!(destination.as_slice(), source);
    assert_eq!(destination.array1().lower(), 5);
}

#[test]
fn raw_rectangle_copy_checks_the_actual_flat_extent_before_reading() {
    let mut destination = Array2::<i32>::with_size(2, 2);
    destination.change_array1().resize(5, false);
    let source = [1, 2, 3, 4];
    let result = unsafe { destination.copy_values_trivial_from_raw_parts(source.as_ptr(), 2, 2) };
    assert!(matches!(
        result,
        Err(ArrayError::DimensionMismatch {
            expected: 5,
            actual: 4
        })
    ));
}

#[test]
fn raw_rectangle_assignment_rejects_invalid_bounds_before_reading() {
    let mut destination = Array2::<i32>::with_size(1, 1);
    let result = unsafe { destination.assign_trivial_from_raw_parts(std::ptr::null(), 3, 1, 0, 0) };
    assert!(matches!(
        result,
        Err(ArrayError::Bounds { lower: 3, upper: 1 })
    ));
    assert_eq!((destination.nb_rows(), destination.nb_columns()), (1, 1));
}

#[test]
fn raw_rectangle_empty_assignment_keeps_empty_dimensions_and_flat_lower() {
    let mut destination = Array2::<i32>::with_size(1, 1);
    unsafe {
        destination
            .assign_trivial_from_raw_parts(std::ptr::null(), -2, -3, 4, 6)
            .unwrap();
    }
    assert_eq!((destination.nb_rows(), destination.nb_columns()), (0, 3));
    assert_eq!(destination.array1().lower(), -2);
    assert!(destination.data_ptr().is_null());
}

#[test]
fn raw_rectangle_dimension_overflow_reports_values_before_reading() {
    let mut destination = Array2::<i32>::default();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        destination
            .assign_trivial_from_raw_parts(std::ptr::null(), i32::MIN, i32::MAX, i32::MIN, i32::MAX)
            .unwrap();
    }));
    let message = result.unwrap_err().downcast::<String>().unwrap();
    assert!(message.contains("4294967296, 4294967296"));
    assert!(destination.is_empty());
}
