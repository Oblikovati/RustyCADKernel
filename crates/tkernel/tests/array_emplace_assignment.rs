//! Native EmplaceValue construction, assignment, temporary destruction and return order.
#[path = "array_emplace_assignment/support.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{EmplaceArray, EmplaceItem, assert_native_row, exercise, values_row};

fn check_owned(mode: i32) {
    for matrix in [false, true] {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut array = EmplaceArray::owned(matrix, &trace);
        let actual = exercise(&mut array, &trace, mode, 0);
        assert_native_row(&format!("owned_{}_{mode}", u8::from(matrix)), &actual);
    }
}

fn check_borrowed(mode: i32) {
    for matrix in [false, true] {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut buffer =
            std::array::from_fn::<_, 5, _>(|index| EmplaceItem::supplied(index as i32 + 1, &trace));
        let mut array = EmplaceArray::borrowed(matrix, &mut buffer);
        let actual = exercise(&mut array, &trace, mode, 0);
        drop(array);
        assert_native_row(
            &format!("borrowed_{}_{mode}", u8::from(matrix)),
            &format!("{actual};{}", values_row(&buffer)),
        );
    }
}

#[test]
fn owned_success_assigns_existing_object_then_destroys_modified_temporary_before_return() {
    check_owned(0);
}
#[test]
fn owned_constructor_failure_preserves_target_without_destroying_unconstructed_temporary() {
    check_owned(1);
}
#[test]
fn owned_assignment_failure_destroys_temporary_and_preserves_target() {
    check_owned(2);
}
#[test]
fn owned_late_assignment_failure_preserves_partial_target_and_destroys_modified_temporary() {
    check_owned(3);
}
#[test]
fn borrowed_success_preserves_external_slot_identity_and_untouched_tail() {
    check_borrowed(0);
}
#[test]
fn borrowed_constructor_failure_keeps_external_buffer_unchanged() {
    check_borrowed(1);
}
#[test]
fn borrowed_assignment_failure_destroys_only_temporary() {
    check_borrowed(2);
}
#[test]
fn borrowed_late_assignment_failure_keeps_external_target_mutation() {
    check_borrowed(3);
}

fn check_invalid(matrix: bool, invalid: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = EmplaceArray::owned(matrix, &trace);
    let actual = exercise(&mut array, &trace, 1, invalid);
    assert_native_row(&format!("invalid_{}_{invalid}", u8::from(matrix)), &actual);
}
#[test]
fn array1_bounds_error_precedes_even_a_throwing_constructor() {
    check_invalid(false, 1);
}
#[test]
fn array2_row_error_precedes_even_a_throwing_constructor() {
    check_invalid(true, 1);
}
#[test]
fn array2_column_error_precedes_even_a_throwing_constructor() {
    check_invalid(true, 2);
}

#[test]
fn array2_shortened_inherited_storage_rejects_after_constructing_and_destroys_temporary() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = EmplaceArray::owned(true, &trace);
    if let EmplaceArray::Matrix(matrix) = &mut array {
        *matrix.change_array1() = rustycad_tkernel::ncollection::Array1::default();
    }
    trace.borrow_mut().clear();
    assert_native_row("short_base", &exercise(&mut array, &trace, 0, 0));
}

#[test]
fn native_copy_assignment_fallback_destroys_an_unmodified_temporary() {
    check_owned(4);
}

#[test]
fn array1_returned_cell_and_once_only_callbacks_support_owned_captures() {
    use rustycad_tkernel::ncollection::Array1;
    let mut array = Array1::<i32>::with_size(1);
    let constructor_capture = Box::new(42);
    let assignment_capture = Box::new(());
    let returned = array
        .emplace_value_with_assignment(
            0,
            move || *std::convert::identity(constructor_capture),
            move |slot, donor| {
                drop(assignment_capture);
                *slot = *donor;
            },
        )
        .unwrap();
    *returned += 1;
    assert_eq!(array.as_slice(), &[43]);
}

#[test]
fn array2_returned_cell_and_once_only_callbacks_support_owned_captures() {
    use rustycad_tkernel::ncollection::Array2;
    let mut array = Array2::<i32>::with_size(1, 1);
    let constructor_capture = Box::new(42);
    let assignment_capture = Box::new(());
    let returned = array
        .emplace_value_with_assignment(
            0,
            0,
            move || *std::convert::identity(constructor_capture),
            move |slot, donor| {
                drop(assignment_capture);
                *slot = *donor;
            },
        )
        .unwrap();
    *returned += 1;
    assert_eq!(array.as_slice(), &[43]);
}
