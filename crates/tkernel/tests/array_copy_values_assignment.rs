//! CopyValues requires copy assignment, not a copy constructor.
#[path = "array_copy_values_assignment/support.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{AssignOnlyItem, CopyArray, assert_native_row, exercise};

fn check_case(matrix: bool, target_owned: bool, source_owned: bool, mode: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut targets =
        std::array::from_fn::<_, 4, _>(|i| AssignOnlyItem::supplied(i as i32 + 1, &trace));
    let mut donors =
        std::array::from_fn::<_, 4, _>(|i| AssignOnlyItem::supplied(i as i32 + 11, &trace));
    let mut target = copy_array_fixture(matrix, target_owned, &mut targets, false, &trace);
    let source = copy_array_fixture(matrix, source_owned, &mut donors, true, &trace);
    let label = format!(
        "case_{}_{}_{}_{mode}",
        u8::from(matrix),
        u8::from(target_owned),
        u8::from(source_owned)
    );
    assert_native_row(&label, &exercise(&mut target, &source, mode, &trace));
}
fn copy_array_fixture<'a>(
    matrix: bool,
    owned: bool,
    buffer: &'a mut [AssignOnlyItem],
    source: bool,
    trace: &support::CopyTrace,
) -> CopyArray<'a> {
    if owned {
        return CopyArray::owned(matrix, source, trace);
    }
    CopyArray::borrowed(matrix, buffer, source)
}
fn check_mode(mode: i32) {
    for matrix in [false, true] {
        check_case(matrix, false, false, mode);
        check_case(matrix, false, true, mode);
        check_case(matrix, true, false, mode);
        check_case(matrix, true, true, mode);
    }
}
#[test]
fn copy_values_assigns_in_order_and_returns_the_unchanged_receiver() {
    check_mode(0);
}
#[test]
fn early_failure_preserves_completed_prefix_and_unchanged_suffix() {
    check_mode(1);
}
#[test]
fn late_failure_retains_partial_assignment_and_never_mutates_donors() {
    check_mode(2);
}
#[test]
fn disjoint_views_of_the_same_allocation_support_const_source_assignment() {
    for matrix in [false, true] {
        check_disjoint(matrix);
    }
}
fn check_disjoint(matrix: bool) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = std::array::from_fn::<_, 8, _>(|i| {
        AssignOnlyItem::supplied(if i < 4 { i as i32 + 1 } else { i as i32 + 7 }, &trace)
    });
    let (targets, donors) = buffer.split_at_mut(4);
    let mut target = CopyArray::borrowed(matrix, targets, false);
    let source = CopyArray::borrowed(matrix, donors, true);
    assert_native_row(
        &format!("disjoint_{}", u8::from(matrix)),
        &exercise(&mut target, &source, 0, &trace),
    );
}

fn check_mismatch(matrix: bool, shape: bool) {
    use rustycad_tkernel::ncollection::Array1;
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut donors =
        std::array::from_fn::<_, 3, _>(|i| AssignOnlyItem::supplied(i as i32 + 11, &trace));
    let mut target = CopyArray::owned(matrix, false, &trace);
    let mut source = CopyArray::owned(matrix, true, &trace);
    match &mut source {
        CopyArray::Linear(array) => *array = Array1::from_slice(&mut donors),
        CopyArray::Matrix(array) => replace_source_storage(array, &mut donors, shape),
    }
    trace.borrow_mut().clear();
    let label = format!("mismatch_{}_{}", u8::from(matrix), u8::from(shape));
    assert_native_row(&label, &exercise(&mut target, &source, 1, &trace));
}
fn replace_source_storage<'a>(
    array: &mut rustycad_tkernel::ncollection::Array2<'a, AssignOnlyItem>,
    donors: &'a mut [AssignOnlyItem],
    shape: bool,
) {
    use rustycad_tkernel::ncollection::{Array1, Array2};
    if shape {
        *array = Array2::with_bounds(7, 7, -4, -1).unwrap();
    }
    *array.change_array1() = Array1::from_slice(donors);
}
#[test]
fn unequal_linear_lengths_fail_before_assignment() {
    check_mismatch(false, false);
}
#[test]
fn unequal_matrix_flat_lengths_fail_after_matching_shapes() {
    check_mismatch(true, false);
}
#[test]
fn unequal_matrix_shapes_fail_before_unequal_flat_lengths() {
    check_mismatch(true, true);
}
fn check_flat(length: usize) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut targets =
        std::array::from_fn::<_, 5, _>(|i| AssignOnlyItem::supplied(i as i32 + 1, &trace));
    let mut donors =
        std::array::from_fn::<_, 5, _>(|i| AssignOnlyItem::supplied(i as i32 + 11, &trace));
    let mut target = CopyArray::owned(true, false, &trace);
    let mut source = CopyArray::owned(true, true, &trace);
    if let CopyArray::Matrix(array) = &mut target {
        replace_source_storage(array, &mut targets[..length], false);
    }
    if let CopyArray::Matrix(array) = &mut source {
        replace_source_storage(array, &mut donors[..length], false);
    }
    trace.borrow_mut().clear();
    assert_native_row(
        &format!("flat_{length}"),
        &exercise(&mut target, &source, 0, &trace),
    );
}
#[test]
fn matrix_copy_values_copies_equal_flat_lengths_larger_than_rectangle() {
    check_flat(5);
}
#[test]
fn matrix_copy_values_copies_equal_flat_lengths_smaller_than_rectangle() {
    check_flat(2);
}
#[test]
fn matrix_copy_values_accepts_empty_inherited_buffers_without_assignment() {
    check_flat(0);
}

struct NoCopyConstruction(i32);
#[test]
fn linear_callback_supports_non_default_types_and_retains_mutable_state() {
    use rustycad_tkernel::ncollection::Array1;
    let mut targets = [NoCopyConstruction(1), NoCopyConstruction(2)];
    let mut donors = [NoCopyConstruction(11), NoCopyConstruction(12)];
    let mut target = Array1::from_slice(&mut targets);
    let source = Array1::from_slice(&mut donors);
    let mut calls = 0;
    target
        .copy_values_with_assignment(&source, |slot, donor| {
            calls += 1;
            slot.0 = donor.0;
        })
        .unwrap();
    assert_eq!(
        (target.as_slice()[0].0, target.as_slice()[1].0, calls),
        (11, 12, 2)
    );
}
#[test]
fn matrix_callback_supports_non_default_types_and_retains_mutable_state() {
    use rustycad_tkernel::ncollection::Array2;
    let mut targets = [NoCopyConstruction(1), NoCopyConstruction(2)];
    let mut donors = [NoCopyConstruction(11), NoCopyConstruction(12)];
    let mut target = Array2::from_buffer(&mut targets, 1, 2).unwrap();
    let source = Array2::from_buffer(&mut donors, 1, 2).unwrap();
    let mut calls = 0;
    target
        .copy_values_with_assignment(&source, |slot, donor| {
            calls += 1;
            slot.0 = donor.0;
        })
        .unwrap();
    assert_eq!(
        (target.as_slice()[0].0, target.as_slice()[1].0, calls),
        (11, 12, 2)
    );
}
