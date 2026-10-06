#![allow(unsafe_code)]
//! Raw native copying never constructs competing references to overlapping windows.
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};

fn assert_native_row(label: &str, value: impl std::fmt::Display) {
    let expected = include_str!("../../../tools/oracle/tkernel_array_raw_copy.expected");
    let row = format!("{label}={value}");
    assert!(
        expected.lines().any(|line| line == row),
        "missing oracle row: {row}"
    );
}

fn raw_overlap_cases(array: &mut Array1<'_, i32>) {
    for (label, destination, source) in [
        ("raw_forward", 2, 0),
        ("raw_backward", 0, 2),
        ("raw_same", 1, 1),
    ] {
        array.as_mut_slice().copy_from_slice(&[0, 1, 2, 3, 4, 5]);
        let pointer = array.data_mut_ptr();
        unsafe {
            Array1::copy_assign_trivial_raw(pointer.add(destination), pointer.add(source), 4);
        }
        assert_native_row(label, csv(array.as_slice()));
    }
}

fn csv(values: &[i32]) -> String {
    values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn borrowed_array1_raw_copy_both_overlap_directions_and_same_pointer() {
    let mut buffer = [0; 6];
    raw_overlap_cases(&mut Array1::from_slice(&mut buffer));
}

#[test]
fn owned_array1_raw_copy_both_overlap_directions_and_same_pointer() {
    raw_overlap_cases(&mut Array1::with_size(6));
}

#[test]
fn array1_copy_values_same_pointer_preserves_bounds_and_ownership() {
    let mut destination = Array1::<i32>::with_bounds(-2, 1).unwrap();
    destination.as_mut_slice().copy_from_slice(&[0, 1, 2, 3]);
    let source = destination.data_mut_ptr();
    let receiver = std::ptr::from_mut(&mut destination);
    let returned = unsafe {
        destination
            .copy_values_trivial_from_raw_parts(source, 4)
            .unwrap()
    };
    assert!(std::ptr::eq(receiver, returned));
    assert_native_row("array1_same_owned", array1_state(&destination));
}

#[test]
fn borrowed_array1_copy_values_same_pointer_does_not_detach() {
    let mut buffer = [0, 1, 2, 3];
    let mut destination = Array1::from_slice(&mut buffer);
    let source = destination.data_mut_ptr();
    unsafe {
        destination
            .copy_values_trivial_from_raw_parts(source, 4)
            .unwrap();
    }
    assert_eq!(destination.data_ptr(), source);
    assert!(!destination.is_deletable());
}

#[test]
fn owned_array1_raw_assign_from_internal_slice() {
    let mut destination = Array1::<i32>::with_size(6);
    destination
        .as_mut_slice()
        .copy_from_slice(&[0, 1, 2, 3, 4, 5]);
    let source = unsafe { destination.data_mut_ptr().add(1) };
    unsafe {
        destination.assign_trivial_from_raw_parts(source, 4, -2);
    }
    assert_native_row("array1_internal_assign", array1_state(&destination));
}

#[test]
fn borrowed_array1_raw_assign_detaches_from_internal_slice() {
    let mut buffer = [0, 1, 2, 3, 4, 5];
    let mut destination = Array1::from_slice(&mut buffer);
    let source = unsafe { destination.data_mut_ptr().add(1) };
    unsafe {
        destination.assign_trivial_from_raw_parts(source, 4, -2);
    }
    assert_native_row("array1_internal_assign", array1_state(&destination));
    drop(destination);
    assert_eq!(buffer, [0, 1, 2, 3, 4, 5]);
}

#[test]
fn array2_raw_assign_from_internal_rectangle() {
    let mut destination = Array2::<i32>::with_size(2, 3);
    destination
        .as_mut_slice()
        .copy_from_slice(&[0, 1, 2, 3, 4, 5]);
    let source = unsafe { destination.data_mut_ptr().add(1) };
    unsafe {
        destination
            .assign_trivial_from_raw_parts(source, -2, -1, 3, 4)
            .unwrap();
    }
    assert_native_row("array2_internal_assign", array2_state(&destination));
    assert_eq!(destination.array1().lower(), -1);
}

#[test]
fn borrowed_array2_raw_assign_reshapes_same_pointer() {
    let mut buffer = [0, 1, 2, 3];
    let mut destination = Array2::from_buffer(&mut buffer, 2, 2).unwrap();
    let source = destination.data_mut_ptr();
    unsafe {
        destination
            .assign_trivial_from_raw_parts(source, 7, 7, 10, 13)
            .unwrap();
    }
    assert_native_row("array2_reshape_same", array2_state(&destination));
    assert_eq!(destination.data_ptr(), source);
    assert_eq!(destination.array1().lower(), 38);
}

#[test]
fn raw_copy_dimension_mismatch_rejects_without_reading_source() {
    let mut destination = Array1::<i32>::with_size(4);
    let error = unsafe { destination.copy_values_trivial_from_raw_parts(std::ptr::null(), 3) };
    assert!(matches!(
        error,
        Err(ArrayError::DimensionMismatch {
            expected: 4,
            actual: 3
        })
    ));
    assert_native_row("array1_mismatch", "dimension");
}

#[test]
fn raw_copy_rectangle_mismatch_rejects_without_reading_source() {
    let mut destination = Array2::<i32>::with_size(2, 2);
    let error = unsafe { destination.copy_values_trivial_from_raw_parts(std::ptr::null(), 1, 4) };
    assert!(matches!(
        error,
        Err(ArrayError::ShapeMismatch {
            expected: (2, 2),
            actual: (1, 4)
        })
    ));
    assert_native_row("array2_mismatch", "shape");
}

#[test]
fn empty_raw_array1_copy_accepts_null_source() {
    let mut destination = Array1::<i32>::with_size(0);
    unsafe {
        destination
            .copy_values_trivial_from_raw_parts(std::ptr::null(), 0)
            .unwrap();
    }
    assert_native_row("array1_empty_copy", empty_array1_state(&destination));
    unsafe {
        Array1::<i32>::copy_assign_trivial_raw(std::ptr::null_mut(), std::ptr::null(), 0);
    }
}

#[test]
fn empty_raw_array2_copy_accepts_null_source() {
    let mut destination = Array2::<i32>::with_size(0, 3);
    unsafe {
        destination
            .copy_values_trivial_from_raw_parts(std::ptr::null(), 0, 3)
            .unwrap();
    }
    assert_native_row("array2_empty_copy", empty_array2_state(&destination));
}

fn array1_state(array: &Array1<'_, i32>) -> String {
    format!(
        "{};{},{};{}",
        csv(array.as_slice()),
        array.lower(),
        array.upper(),
        u8::from(array.is_deletable())
    )
}

fn array2_state(array: &Array2<'_, i32>) -> String {
    format!(
        "{};{},{},{},{};{}",
        csv(array.as_slice()),
        array.lower_row(),
        array.upper_row(),
        array.lower_col(),
        array.upper_col(),
        u8::from(array.is_deletable())
    )
}

fn empty_array1_state(array: &Array1<'_, i32>) -> String {
    format!(
        "{},{};{};{}",
        array.lower(),
        array.upper(),
        u8::from(array.data_ptr().is_null()),
        u8::from(array.is_deletable())
    )
}

fn empty_array2_state(array: &Array2<'_, i32>) -> String {
    format!(
        "{},{};{};{}",
        array.nb_rows(),
        array.nb_columns(),
        u8::from(array.data_ptr().is_null()),
        u8::from(array.is_deletable())
    )
}
