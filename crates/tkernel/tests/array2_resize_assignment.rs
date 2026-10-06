//! Native Array2 receiver publication and mutable-donor assignment phases.
#[path = "array2_resize_assignment/support.rs"]
mod support;
use rustycad_tkernel::ncollection::Array2;
use std::panic::{AssertUnwindSafe, catch_unwind};
use support::{
    ResizeItem, ResizeTraceScope, assert_native_row, assign_move, initialized_owned, resize_copy,
    state, values_row,
};

#[test]
fn owned_resize_moves_the_packed_prefix_before_destroying_old_elements() {
    check_owned("owned_packed", false, 0);
}

#[test]
fn owned_trim_moves_the_common_rectangle_before_destroying_old_elements() {
    check_owned("owned_trim", true, 0);
}

#[test]
fn owned_assignment_failure_retains_published_shape_and_partial_values() {
    check_owned("owned_assignment_failure", false, 3);
}

fn check_owned(label: &str, trim: bool, failure: i32) {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    scope.clear(failure);
    let result = catch_unwind(AssertUnwindSafe(|| resize_copy(&mut array, trim)));
    assert_native_row(
        label,
        format!(
            "{};{};{}",
            state(&array),
            u8::from(result.is_err()),
            scope.row()
        ),
    );
}

#[test]
fn borrowed_resize_mutates_packed_donors_without_destroying_external_elements() {
    check_borrowed("borrowed_packed", false, 0);
}

#[test]
fn borrowed_trim_mutates_only_the_common_rectangle_donors() {
    check_borrowed("borrowed_trim", true, 0);
}

#[test]
fn borrowed_assignment_failure_keeps_modified_donors_and_new_receiver() {
    check_borrowed("borrowed_assignment_failure", true, 4);
}

fn check_borrowed(label: &str, trim: bool, failure: i32) {
    let scope = ResizeTraceScope::new();
    let mut buffer: [ResizeItem; 7] =
        std::array::from_fn(|index| ResizeItem::supplied(index as i32 + 1, &scope.trace));
    let mut array = Array2::from_buffer_bounds(&mut buffer, -2, -1, 3, 5).unwrap();
    scope.clear(failure);
    let result = catch_unwind(AssertUnwindSafe(|| resize_copy(&mut array, trim)));
    let prefix = format!(
        "{};{};{}",
        state(&array),
        u8::from(result.is_err()),
        scope.row()
    );
    drop(array);
    assert_native_row(label, format!("{prefix};{}", values_row(&buffer)));
}

#[test]
fn same_area_no_copy_keeps_storage_and_skips_all_element_callbacks() {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    let pointer = array.data_ptr();
    scope.clear(0);
    array
        .resize_bounds_with_assignment(7, 9, -3, -2, false, assign_move)
        .unwrap();
    assert_eq!(array.data_ptr(), pointer);
    assert_native_row(
        "same_area_no_copy",
        format!("{};{}", state(&array), scope.row()),
    );
}

#[test]
fn unchanged_dimensions_rebase_without_construction_or_assignment() {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    scope.clear(0);
    array
        .resize_trim_bounds_with_assignment(7, 8, -3, -1, true, assign_move)
        .unwrap();
    assert_native_row(
        "same_dimensions",
        format!("{};{}", state(&array), scope.row()),
    );
}

#[test]
fn zero_counts_clear_owned_elements_without_running_assignments() {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    scope.clear(0);
    array
        .resize_with_assignment(0, 4, true, assign_move)
        .unwrap();
    assert_native_row("zero_count", format!("{};{}", state(&array), scope.row()));
}

#[test]
fn empty_source_default_constructs_without_assigning() {
    let scope = ResizeTraceScope::new();
    let mut array = Array2::<ResizeItem>::with_size(0, 3);
    scope.clear(0);
    array
        .resize_trim_bounds_with_assignment(7, 8, -3, -2, true, assign_move)
        .unwrap();
    assert_native_row("empty_source", format!("{};{}", state(&array), scope.row()));
}

#[test]
fn unchanged_dimensions_keep_the_inherited_arrays_actual_length() {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    array.change_array1().resize(7, false);
    for (index, item) in array.as_mut_slice().iter_mut().enumerate() {
        item.value = index as i32 + 1;
    }
    scope.clear(0);
    array.resize_bounds(7, 8, -3, -1, true).unwrap();
    assert_native_row(
        "retained_flat_length",
        format!("{};{}", state(&array), scope.row()),
    );
}

#[test]
fn copy_fallback_failure_retains_the_native_partial_receiver_state() {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    scope.clear(3);
    let result = catch_unwind(AssertUnwindSafe(|| array.resize_bounds(7, 8, -3, -2, true)));
    assert_native_row(
        "copy_fallback_failure",
        format!(
            "{};{};{}",
            state(&array),
            u8::from(result.is_err()),
            scope.row()
        ),
    );
}

#[test]
fn count_resize_preserves_origins_with_packed_donor_order() {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    scope.clear(0);
    array
        .resize_with_assignment(2, 2, true, assign_move)
        .unwrap();
    assert_native_row(
        "count_resize_0",
        format!("{};{}", state(&array), scope.row()),
    );
}

#[test]
fn count_trim_preserves_origins_with_row_column_donor_order() {
    let scope = ResizeTraceScope::new();
    let mut array = initialized_owned();
    scope.clear(0);
    array
        .resize_trim_with_assignment(2, 2, true, assign_move)
        .unwrap();
    assert_native_row(
        "count_resize_1",
        format!("{};{}", state(&array), scope.row()),
    );
}

#[test]
fn shared_copy_fallback_never_detaches_or_copy_constructs_its_donors() {
    let scope = ResizeTraceScope::new();
    let buffer: [ResizeItem; 6] =
        std::array::from_fn(|index| ResizeItem::supplied(index as i32 + 1, &scope.trace));
    let mut array = Array2::from_shared(&buffer, -2, -1, 3, 5).unwrap();
    scope.clear(0);
    array.resize_trim_bounds(7, 8, -3, -2, true).unwrap();
    assert_native_row(
        "shared_copy_fallback",
        format!("{};{};{}", state(&array), scope.row(), values_row(&buffer)),
    );
}
