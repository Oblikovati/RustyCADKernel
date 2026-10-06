//! Native Init(array.Value(index)) reads a live donor, including self-assignment.
#[path = "array_internal_init/support.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{InternalDonorItem, InternalInitArray, assert_native_row, exercise};

fn check_internal_case(matrix: bool, owned: bool, offset: i32, mode: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer =
        std::array::from_fn::<_, 6, _>(|i| InternalDonorItem::supplied(i as i32 + 1, &trace));
    let mut array = if owned {
        InternalInitArray::owned(matrix, &trace)
    } else {
        InternalInitArray::borrowed(matrix, &mut buffer)
    };
    let label = format!(
        "case_{}_{}_{offset}_{mode}",
        u8::from(matrix),
        u8::from(owned)
    );
    assert_native_row(&label, &exercise(&mut array, offset, mode, 0, &trace));
}
fn check_internal_mode(mode: i32) {
    for matrix in [false, true] {
        check_internal_case(matrix, false, 2, mode);
        check_internal_case(matrix, true, 2, mode);
    }
}
#[test]
fn native_self_assignment_changes_the_donor_seen_by_later_slots() {
    check_internal_mode(0);
}
#[test]
fn prefix_failure_skips_the_later_self_assignment() {
    check_internal_mode(1);
}
#[test]
fn early_self_failure_retains_prefix_without_mutating_the_donor() {
    check_internal_mode(2);
}
#[test]
fn late_self_failure_retains_modified_donor_and_untouched_suffix() {
    check_internal_mode(3);
}
#[test]
fn suffix_early_failure_retains_completed_self_assignment() {
    check_internal_mode(4);
}
#[test]
fn suffix_late_failure_retains_failing_target_mutation() {
    check_internal_mode(5);
}
#[test]
fn first_and_last_donors_support_empty_prefix_or_suffix() {
    for matrix in [false, true] {
        check_internal_case(matrix, false, 0, 0);
        check_internal_case(matrix, true, 0, 0);
        check_internal_case(matrix, false, 5, 0);
        check_internal_case(matrix, true, 5, 0);
    }
}
fn check_invalid(matrix: bool, invalid: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = InternalInitArray::owned(matrix, &trace);
    assert_native_row(
        &format!("invalid_{}_{invalid}", u8::from(matrix)),
        &exercise(&mut array, 2, 1, invalid, &trace),
    );
}
#[test]
fn invalid_source_indices_precede_all_assignment_callbacks() {
    check_invalid(false, 1);
    check_invalid(true, 1);
    check_invalid(true, 2);
}
#[test]
fn empty_arrays_have_no_valid_internal_donor() {
    use rustycad_tkernel::ncollection::{Array1, Array2};
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut first = InternalInitArray::Linear(Array1::default());
    let mut second = InternalInitArray::Matrix(Array2::default());
    assert_native_row("empty_0", &exercise(&mut first, 0, 0, 0, &trace));
    assert_native_row("empty_1", &exercise(&mut second, 0, 0, 0, &trace));
}
fn check_flat(length: usize, offset: i32) {
    use rustycad_tkernel::ncollection::Array1;
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer =
        std::array::from_fn::<_, 8, _>(|i| InternalDonorItem::supplied(i as i32 + 1, &trace));
    let mut array = InternalInitArray::owned(true, &trace);
    if let InternalInitArray::Matrix(matrix) = &mut array {
        *matrix.change_array1() = Array1::from_slice(&mut buffer[..length]);
    }
    trace.borrow_mut().clear();
    assert_native_row(
        &format!("flat_{length}"),
        &exercise(&mut array, offset, 0, 0, &trace),
    );
}
#[test]
fn internal_matrix_donor_initializes_extended_inherited_storage() {
    check_flat(8, 2);
}
#[test]
fn internal_matrix_donor_initializes_shortened_valid_inherited_storage() {
    check_flat(2, 0);
}
struct NoConstructionTraits(i32);
fn assign_internal_slot(target: &mut NoConstructionTraits, donor: &NoConstructionTraits) {
    target.0 = donor.0;
}
fn assign_internal_self() -> impl FnOnce(&mut NoConstructionTraits) {
    let capture = Box::new(());
    move |slot| {
        drop(capture);
        slot.0 += 100;
    }
}
#[test]
fn linear_internal_init_accepts_non_default_types_and_once_only_self_callback() {
    use rustycad_tkernel::ncollection::Array1;
    let mut buffer = [
        NoConstructionTraits(1),
        NoConstructionTraits(2),
        NoConstructionTraits(3),
    ];
    let mut array = Array1::from_slice(&mut buffer);
    array
        .init_from_index_with_assignment(1, assign_internal_slot, assign_internal_self())
        .unwrap();
    assert_eq!(
        array
            .as_slice()
            .iter()
            .map(|item| item.0)
            .collect::<Vec<_>>(),
        [2, 102, 102]
    );
}
#[test]
fn matrix_internal_init_accepts_non_default_types_and_once_only_self_callback() {
    use rustycad_tkernel::ncollection::Array2;
    let mut buffer = [
        NoConstructionTraits(1),
        NoConstructionTraits(2),
        NoConstructionTraits(3),
    ];
    let mut array = Array2::from_buffer(&mut buffer, 1, 3).unwrap();
    array
        .init_from_index_with_assignment(0, 1, assign_internal_slot, assign_internal_self())
        .unwrap();
    assert_eq!(
        array
            .as_slice()
            .iter()
            .map(|item| item.0)
            .collect::<Vec<_>>(),
        [2, 102, 102]
    );
}

#[test]
fn matrix_checked_extension_rejects_a_donor_outside_shortened_backing() {
    use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
    let mut array = Array2::<i32>::with_size(1, 2);
    *array.change_array1() = Array1::default();
    let result =
        array.init_from_index_with_assignment(0, 1, |_, _| unreachable!(), |_| unreachable!());
    assert_eq!(
        result,
        Err(ArrayError::Index {
            index: 1,
            lower: 0,
            upper: -1
        })
    );
    assert_eq!(array.len(), 0);
}
