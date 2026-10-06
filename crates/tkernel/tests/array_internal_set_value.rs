//! Native internal SetValue donors remain live; same-element assignment is explicit.
#[path = "array_internal_set_value/support.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{InternalSetArray, InternalSetItem, InternalSetRequest, assert_native_row, exercise};

fn check_internal_case(matrix: bool, owned: bool, request: InternalSetRequest) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer =
        std::array::from_fn::<_, 6, _>(|i| InternalSetItem::supplied(i as i32 + 1, &trace));
    let mut array = if owned {
        InternalSetArray::owned(matrix, &trace)
    } else {
        InternalSetArray::borrowed(matrix, &mut buffer)
    };
    let label = case_label(matrix, owned, request);
    assert_native_row(&label, &exercise(&mut array, request, &trace));
}
fn case_label(matrix: bool, owned: bool, request: InternalSetRequest) -> String {
    format!(
        "case_{}_{}_{}_{}_{}_{}",
        u8::from(matrix),
        u8::from(owned),
        u8::from(request.moved),
        request.target,
        request.source,
        request.mode
    )
}
fn check_storage_cases(matrix: bool, mode: i32, moved: bool) {
    for (target, source) in [(0, 2), (2, 0), (2, 2)] {
        let request = InternalSetRequest {
            target,
            source,
            moved,
            mode,
            invalid: 0,
        };
        check_internal_case(matrix, false, request);
        check_internal_case(matrix, true, request);
    }
}
fn check_internal_mode(mode: i32) {
    for matrix in [false, true] {
        check_storage_cases(matrix, mode, false);
        check_storage_cases(matrix, mode, true);
    }
}
#[test]
fn distinct_copy_move_and_self_assignment_preserve_native_live_donor_behavior() {
    check_internal_mode(0);
}
#[test]
fn early_assignment_failures_preserve_target_and_source_values() {
    check_internal_mode(1);
}
#[test]
fn late_assignment_failures_retain_distinct_or_self_mutations() {
    check_internal_mode(2);
}
fn check_invalid(matrix: bool, moved: bool, invalid: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = InternalSetArray::owned(matrix, &trace);
    let request = InternalSetRequest {
        target: 0,
        source: 2,
        moved,
        mode: 1,
        invalid,
    };
    let label = format!("invalid_{}_{}_{invalid}", u8::from(matrix), u8::from(moved));
    assert_native_row(&label, &exercise(&mut array, request, &trace));
}
#[test]
fn source_validation_precedes_target_validation_for_both_overloads() {
    for moved in [false, true] {
        for invalid in 1..=3 {
            check_invalid(false, moved, invalid);
        }
        for invalid in 1..=6 {
            check_invalid(true, moved, invalid);
        }
    }
}

struct NoInternalSetTraits(i32);
fn owned_copy_callback() -> impl FnOnce(&mut NoInternalSetTraits, &NoInternalSetTraits) {
    let capture = Box::new(());
    move |target, source| {
        drop(capture);
        target.0 = source.0;
    }
}
fn owned_self_callback() -> impl FnOnce(&mut NoInternalSetTraits) {
    let capture = Box::new(());
    move |slot| {
        drop(capture);
        slot.0 = -slot.0;
    }
}
#[test]
fn linear_internal_set_accepts_non_default_non_clone_types_and_owned_callbacks() {
    use rustycad_tkernel::ncollection::Array1;
    let mut buffer = [NoInternalSetTraits(1), NoInternalSetTraits(2)];
    let mut array = Array1::from_slice(&mut buffer);
    array
        .set_value_from_index_with_assignment(0, 1, owned_copy_callback(), |_| unreachable!())
        .unwrap();
    array
        .set_value_moved_from_index_with_assignment(
            0,
            0,
            |_, _| unreachable!(),
            owned_self_callback(),
        )
        .unwrap();
    assert_eq!((array.as_slice()[0].0, array.as_slice()[1].0), (-2, 2));
}
#[test]
fn matrix_internal_set_accepts_non_default_non_clone_types_and_owned_callbacks() {
    use rustycad_tkernel::ncollection::Array2;
    let mut buffer = [NoInternalSetTraits(1), NoInternalSetTraits(2)];
    let mut array = Array2::from_buffer(&mut buffer, 1, 2).unwrap();
    array
        .set_value_from_index_with_assignment(
            (0, 0),
            (0, 1),
            owned_copy_callback(),
            |_| unreachable!(),
        )
        .unwrap();
    assert_eq!((array.as_slice()[0].0, array.as_slice()[1].0), (2, 2));
}
#[test]
fn matrix_internal_self_move_accepts_a_non_default_type_and_owned_callback() {
    use rustycad_tkernel::ncollection::Array2;
    let mut buffer = [NoInternalSetTraits(1)];
    let mut array = Array2::from_buffer(&mut buffer, 1, 1).unwrap();
    array
        .set_value_moved_from_index_with_assignment(
            (0, 0),
            (0, 0),
            |_, _| unreachable!(),
            owned_self_callback(),
        )
        .unwrap();
    assert_eq!(array.as_slice()[0].0, -1);
}

fn check_flat_guard(target: i32, source: i32, expected: i64, moved: bool) {
    use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
    let mut array = Array2::<i32>::with_size(1, 3);
    *array.change_array1() = Array1::with_size(1);
    let result = if moved {
        guard_matrix_move(&mut array, (0, target), (0, source))
    } else {
        guard_matrix_copy(&mut array, (0, target), (0, source))
    };
    assert_eq!(
        result,
        Err(ArrayError::Index {
            index: expected,
            lower: 0,
            upper: 0
        })
    );
}
#[test]
fn checked_extension_validates_source_then_target_in_shortened_matrix_storage() {
    for moved in [false, true] {
        check_flat_guard(0, 2, 2, moved);
        check_flat_guard(2, 0, 2, moved);
        check_flat_guard(2, 1, 1, moved);
    }
}

fn guard_matrix_copy(
    array: &mut rustycad_tkernel::ncollection::Array2<'_, i32>,
    target: (i32, i32),
    source: (i32, i32),
) -> Result<(), rustycad_tkernel::ncollection::ArrayError> {
    array.set_value_from_index_with_assignment(
        target,
        source,
        |_, _| unreachable!(),
        |_| unreachable!(),
    )
}
fn guard_matrix_move(
    array: &mut rustycad_tkernel::ncollection::Array2<'_, i32>,
    target: (i32, i32),
    source: (i32, i32),
) -> Result<(), rustycad_tkernel::ncollection::ArrayError> {
    array.set_value_moved_from_index_with_assignment(
        target,
        source,
        |_, _| unreachable!(),
        |_| unreachable!(),
    )
}
