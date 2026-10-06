//! Native const assignment accepts types with no copy constructor.
#[path = "array_const_assignment/support.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{ConstArray, CopyAssignmentOnly, assert_native_row, exercise};

fn check_case(matrix: bool, owned: bool, mode: i32, init: bool) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = std::array::from_fn::<_, 4, _>(|index| {
        CopyAssignmentOnly::supplied(index as i32 + 1, &trace)
    });
    let mut array = if owned {
        ConstArray::owned(matrix, &trace)
    } else {
        ConstArray::borrowed(matrix, &mut buffer)
    };
    let donor = CopyAssignmentOnly::supplied(9, &trace);
    let label = case_label(matrix, owned, mode, init);
    assert_native_row(&label, &exercise(&mut array, &donor, mode, init, 0));
    drop(donor);
    assert_native_row(&format!("{label}_dropped"), &support::trace_row(&trace));
}
fn case_label(matrix: bool, owned: bool, mode: i32, init: bool) -> String {
    format!(
        "case_{}_{}_{}_{mode}",
        u8::from(matrix),
        u8::from(owned),
        u8::from(init)
    )
}
fn check_mode(mode: i32, init: bool) {
    for matrix in [false, true] {
        check_case(matrix, false, mode, init);
        check_case(matrix, true, mode, init);
    }
}
#[test]
fn set_value_assigns_without_constructing_or_destroying_target_or_donor() {
    check_mode(0, false);
}
#[test]
fn set_value_early_failure_preserves_target_and_donor() {
    check_mode(1, false);
}
#[test]
fn set_value_late_failure_preserves_target_mutation_and_donor() {
    check_mode(2, false);
}
#[test]
fn init_assigns_all_slots_in_native_storage_order() {
    check_mode(0, true);
}
#[test]
fn init_early_failure_preserves_completed_prefix_and_untouched_suffix() {
    check_mode(1, true);
}
#[test]
fn init_late_failure_preserves_failing_slot_mutation() {
    check_mode(2, true);
}
fn check_disjoint(matrix: bool, init: bool) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = std::array::from_fn::<_, 5, _>(|index| {
        CopyAssignmentOnly::supplied(index as i32 + 1, &trace)
    });
    buffer[4].value = 9;
    let (targets, tail) = buffer.split_at_mut(4);
    let mut array = ConstArray::borrowed(matrix, targets);
    let actual = exercise(&mut array, &tail[0], 0, init, 0);
    assert_native_row(
        &format!("disjoint_{}_{}", u8::from(matrix), u8::from(init)),
        &actual,
    );
}
#[test]
fn source_in_disjoint_buffer_tail_stays_unchanged_during_const_assignment() {
    for matrix in [false, true] {
        check_disjoint(matrix, false);
        check_disjoint(matrix, true);
    }
}
fn check_invalid(matrix: bool, invalid: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = ConstArray::owned(matrix, &trace);
    let donor = CopyAssignmentOnly::supplied(9, &trace);
    assert_native_row(
        &format!("invalid_{}_{invalid}", u8::from(matrix)),
        &exercise(&mut array, &donor, 1, false, invalid),
    );
}
#[test]
fn invalid_set_value_indices_skip_assignment_before_any_failure_hook() {
    check_invalid(false, 1);
    check_invalid(true, 1);
    check_invalid(true, 2);
}
fn check_flat_extent(length: usize) {
    use rustycad_tkernel::ncollection::Array1;
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = std::array::from_fn::<_, 5, _>(|index| {
        CopyAssignmentOnly::supplied(index as i32 + 1, &trace)
    });
    let mut array = ConstArray::owned(true, &trace);
    if let ConstArray::Matrix(matrix) = &mut array {
        *matrix.change_array1() = Array1::from_slice(&mut buffer[..length]);
    }
    let donor = CopyAssignmentOnly::supplied(9, &trace);
    trace.borrow_mut().clear();
    assert_native_row(
        &format!("flat_{length}"),
        &exercise(&mut array, &donor, 0, true, 0),
    );
}
#[test]
fn matrix_init_uses_inherited_flat_length_beyond_rectangle_area() {
    check_flat_extent(5);
}
#[test]
fn matrix_init_uses_shortened_inherited_flat_length() {
    check_flat_extent(2);
}
#[test]
fn matrix_init_empty_backing_skips_callbacks_without_changing_rectangle() {
    check_flat_extent(0);
}

struct NoConstructionTraits(i32);
fn once_assignment() -> impl FnOnce(&mut NoConstructionTraits, &NoConstructionTraits) {
    let capture = Box::new(());
    move |slot, donor| {
        drop(capture);
        slot.0 = donor.0;
    }
}
#[test]
fn array1_const_policies_accept_non_default_types_and_once_or_mutable_callbacks() {
    use rustycad_tkernel::ncollection::Array1;
    let mut slots = [NoConstructionTraits(1), NoConstructionTraits(2)];
    let donor = NoConstructionTraits(9);
    let mut array = Array1::from_slice(&mut slots);
    array
        .set_value_ref_with_assignment(0, &donor, once_assignment())
        .unwrap();
    let mut calls = 0;
    array.init_ref_with_assignment(&donor, |slot, source| {
        calls += 1;
        slot.0 = source.0;
    });
    assert_eq!(
        (array.as_slice()[0].0, array.as_slice()[1].0, calls),
        (9, 9, 2)
    );
}
#[test]
fn array2_const_policies_accept_non_default_types_and_once_or_mutable_callbacks() {
    use rustycad_tkernel::ncollection::Array2;
    let mut slots = [NoConstructionTraits(1), NoConstructionTraits(2)];
    let donor = NoConstructionTraits(9);
    let mut array = Array2::from_buffer(&mut slots, 1, 2).unwrap();
    array
        .set_value_ref_with_assignment(0, 0, &donor, once_assignment())
        .unwrap();
    let mut calls = 0;
    array.init_ref_with_assignment(&donor, |slot, source| {
        calls += 1;
        slot.0 = source.0;
    });
    assert_eq!(
        (array.as_slice()[0].0, array.as_slice()[1].0, calls),
        (9, 9, 2)
    );
}
