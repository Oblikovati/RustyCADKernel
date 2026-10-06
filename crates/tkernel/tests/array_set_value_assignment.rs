//! Native SetValue(T&&) keeps the source alive and assigns into an existing object.
#[path = "array_set_value_assignment/support.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{AssignmentArray, AssignmentItem, assert_native_row, exercise, trace_row};

fn check_external(matrix: bool, owned: bool, mode: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer =
        std::array::from_fn::<_, 4, _>(|index| AssignmentItem::supplied(index as i32 + 1, &trace));
    let mut array = if owned {
        AssignmentArray::owned(matrix, &trace)
    } else {
        AssignmentArray::borrowed(matrix, &mut buffer)
    };
    let mut donor = AssignmentItem::supplied(9, &trace);
    let label = format!("external_{}_{}_{mode}", u8::from(matrix), u8::from(owned));
    assert_native_row(&label, &exercise(&mut array, &mut donor, mode, 0));
    drop(donor);
    assert_native_row(&format!("{label}_dropped"), &trace_row(&trace));
}
fn check_mode(mode: i32) {
    for matrix in [false, true] {
        check_external(matrix, false, mode);
        check_external(matrix, true, mode);
        check_disjoint(matrix, mode);
    }
}
fn check_disjoint(matrix: bool, mode: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer =
        std::array::from_fn::<_, 5, _>(|index| AssignmentItem::supplied(index as i32 + 1, &trace));
    buffer[4].value = 9;
    let (targets, tail) = buffer.split_at_mut(4);
    let mut array = AssignmentArray::borrowed(matrix, targets);
    let actual = exercise(&mut array, &mut tail[0], mode, 0);
    assert_native_row(&format!("disjoint_{}_{mode}", u8::from(matrix)), &actual);
}
#[test]
fn successful_move_assignment_keeps_target_identity_and_retains_mutated_donor() {
    check_mode(0);
}
#[test]
fn early_assignment_failure_keeps_both_objects_alive_and_unchanged() {
    check_mode(1);
}
#[test]
fn late_assignment_failure_keeps_both_objects_alive_and_mutated() {
    check_mode(2);
}
#[test]
fn native_copy_assignment_fallback_keeps_donor_value_and_lifetime() {
    check_mode(3);
}
fn check_bounds(matrix: bool, invalid: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = AssignmentArray::owned(matrix, &trace);
    let mut donor = AssignmentItem::supplied(9, &trace);
    let label = format!("invalid_{}_{invalid}", u8::from(matrix));
    assert_native_row(&label, &exercise(&mut array, &mut donor, 1, invalid));
    drop(donor);
    assert_native_row(&format!("{label}_dropped"), &trace_row(&trace));
}
#[test]
fn array1_lower_and_upper_errors_do_not_assign_or_destroy_donor() {
    check_bounds(false, 1);
    check_bounds(false, 2);
}
#[test]
fn array2_row_errors_do_not_assign_or_destroy_donor() {
    check_bounds(true, 1);
    check_bounds(true, 2);
}
#[test]
fn array2_column_errors_do_not_assign_or_destroy_donor() {
    check_bounds(true, 3);
    check_bounds(true, 4);
}

struct NoAssignmentTraits(i32);
#[test]
fn array1_accepts_once_only_callback_without_clone_or_default() {
    use rustycad_tkernel::ncollection::Array1;
    let mut slots = [NoAssignmentTraits(2)];
    let mut donor = NoAssignmentTraits(9);
    let capture = Box::new(());
    let mut array = Array1::from_slice(&mut slots);
    array
        .set_value_with_assignment(0, &mut donor, move |slot, source| {
            drop(capture);
            std::mem::swap(slot, source);
        })
        .unwrap();
    assert_eq!((array.as_slice()[0].0, donor.0), (9, 2));
}
#[test]
fn array2_accepts_once_only_callback_without_clone_or_default() {
    use rustycad_tkernel::ncollection::Array2;
    let mut slots = [NoAssignmentTraits(2)];
    let mut donor = NoAssignmentTraits(9);
    let capture = Box::new(());
    let mut array = Array2::from_buffer(&mut slots, 1, 1).unwrap();
    array
        .set_value_with_assignment(0, 0, &mut donor, move |slot, source| {
            drop(capture);
            std::mem::swap(slot, source);
        })
        .unwrap();
    assert_eq!((array.as_slice()[0].0, donor.0), (9, 2));
}
