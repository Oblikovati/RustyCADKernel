//! Native internal constructor arguments outlive the distinct EmplaceValue temporary.
#[path = "array_internal_emplace/support.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{
    InternalEmplaceArray, InternalEmplaceItem, InternalEmplaceRequest, assert_native_row, exercise,
};

fn check_internal_case(matrix: bool, owned: bool, request: InternalEmplaceRequest) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer =
        std::array::from_fn::<_, 6, _>(|i| InternalEmplaceItem::supplied(i as i32 + 1, &trace));
    let mut array = if owned {
        InternalEmplaceArray::owned(matrix, &trace)
    } else {
        InternalEmplaceArray::borrowed(matrix, &mut buffer)
    };
    let label = case_label(matrix, owned, request);
    assert_native_row(&label, &exercise(&mut array, request, &trace));
}
fn case_label(matrix: bool, owned: bool, request: InternalEmplaceRequest) -> String {
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
        let request = InternalEmplaceRequest {
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
fn internal_copy_and_move_arguments_construct_distinct_temporaries() {
    check_internal_mode(0);
}
#[test]
fn early_constructor_failures_preserve_target_and_source_values() {
    check_internal_mode(1);
}
#[test]
fn late_constructor_failures_retain_mutable_source_changes() {
    check_internal_mode(2);
}
fn check_invalid(matrix: bool, moved: bool, invalid: i32) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = InternalEmplaceArray::owned(matrix, &trace);
    let request = InternalEmplaceRequest {
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

#[test]
fn early_assignment_failure_destroys_the_constructed_temporary() {
    check_internal_mode(3);
}
#[test]
fn late_assignment_failure_preserves_target_changes_and_destroys_temporary() {
    check_internal_mode(4);
}
fn shortened_matrix<'a>(trace: &support::InternalEmplaceTrace) -> InternalEmplaceArray<'a> {
    use rustycad_tkernel::ncollection::Array1;
    let mut array = InternalEmplaceArray::owned(true, trace);
    let InternalEmplaceArray::Matrix(matrix) = &mut array else {
        unreachable!()
    };
    *matrix.change_array1() = Array1::with_size(1);
    matrix.change_array1().as_mut_slice()[0] = InternalEmplaceItem::supplied(1, trace);
    trace.borrow_mut().clear();
    array
}
fn short_request(moved: bool, source: i32) -> InternalEmplaceRequest {
    InternalEmplaceRequest {
        target: 2,
        source,
        moved,
        mode: 0,
        invalid: 0,
    }
}
fn check_short_backing(moved: bool) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut array = shortened_matrix(&trace);
    assert!(array.assign(short_request(moved, 0)).is_err());
    trace.borrow_mut().push("E".into());
    let actual = format!(
        "{};{}",
        support::values_row(array.values()),
        trace.borrow().join(",")
    );
    assert_native_row(&format!("short_{}", u8::from(moved)), &actual);
}
#[test]
fn matrix_target_flat_validation_follows_constructor_and_precedes_assignment() {
    check_short_backing(false);
    check_short_backing(true);
}

#[test]
fn shortened_matrix_source_guard_skips_construction_as_a_checked_extension() {
    for moved in [false, true] {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut array = shortened_matrix(&trace);
        assert!(array.assign(short_request(moved, 1)).is_err());
        assert!(trace.borrow().is_empty());
        assert_eq!(support::values_row(array.values()), "1");
    }
}

struct NoEmplaceTraits(i32);
fn owned_copy_constructor() -> impl FnOnce(&NoEmplaceTraits) -> NoEmplaceTraits {
    let capture = Box::new(());
    move |source| {
        drop(capture);
        NoEmplaceTraits(source.0 * 10)
    }
}
fn owned_move_constructor() -> impl FnOnce(&mut NoEmplaceTraits) -> NoEmplaceTraits {
    let capture = Box::new(());
    move |source| {
        drop(capture);
        NoEmplaceTraits(std::mem::take(&mut source.0))
    }
}
fn owned_temporary_assignment() -> impl FnOnce(&mut NoEmplaceTraits, &mut NoEmplaceTraits) {
    let capture = Box::new(());
    move |target, temporary| {
        drop(capture);
        std::mem::swap(target, temporary);
    }
}
#[test]
fn linear_internal_emplace_accepts_owned_callbacks_without_default_or_clone() {
    use rustycad_tkernel::ncollection::Array1;
    let mut buffer = [NoEmplaceTraits(1), NoEmplaceTraits(2)];
    let mut array = Array1::from_slice(&mut buffer);
    let construct = owned_copy_constructor();
    let assign = owned_temporary_assignment();
    array
        .emplace_value_from_index_with_assignment(0, 1, construct, assign)
        .unwrap();
    let construct = owned_move_constructor();
    let assign = owned_temporary_assignment();
    array
        .emplace_value_moved_from_index_with_assignment(0, 0, construct, assign)
        .unwrap();
    assert_eq!((array.as_slice()[0].0, array.as_slice()[1].0), (20, 2));
}
#[test]
fn matrix_internal_emplace_accepts_owned_callbacks_without_default_or_clone() {
    use rustycad_tkernel::ncollection::Array2;
    let mut buffer = [NoEmplaceTraits(1), NoEmplaceTraits(2)];
    let mut array = Array2::from_buffer(&mut buffer, 1, 2).unwrap();
    let construct = owned_copy_constructor();
    let assign = owned_temporary_assignment();
    array
        .emplace_value_from_index_with_assignment((0, 0), (0, 1), construct, assign)
        .unwrap();
    let construct = owned_move_constructor();
    let assign = owned_temporary_assignment();
    array
        .emplace_value_moved_from_index_with_assignment((0, 0), (0, 0), construct, assign)
        .unwrap();
    assert_eq!((array.as_slice()[0].0, array.as_slice()[1].0), (20, 2));
}
