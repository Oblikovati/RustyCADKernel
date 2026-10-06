//! Widened native offsets remain defined even when an unused upper-bound getter would overflow.
#[path = "array2_index_limits/support.rs"]
mod support;
use support::check_case;

#[test]
fn row_origin_near_maximum_preserves_reachable_index_access() {
    check_case("row_first", (i32::MAX - 1, 0), (i32::MAX - 1, 0));
    check_case("row_last_reachable", (i32::MAX - 1, 0), (i32::MAX, 1));
}
#[test]
fn column_origin_near_maximum_preserves_reachable_index_access() {
    check_case("col_first", (0, i32::MAX - 1), (0, i32::MAX - 1));
    check_case("col_last_reachable", (0, i32::MAX - 1), (1, i32::MAX));
}
#[test]
fn both_origins_near_maximum_preserve_reachable_index_access() {
    let lower = (i32::MAX - 1, i32::MAX - 1);
    check_case("both_first", lower, lower);
    check_case("both_last_reachable", lower, (i32::MAX, i32::MAX));
}
#[test]
fn negative_and_minimum_origins_use_signed_widened_offsets() {
    check_case("negative_valid", (-2, -3), (-1, -2));
    check_case(
        "minimum_valid",
        (i32::MIN, i32::MIN),
        (i32::MIN + 1, i32::MIN + 1),
    );
}
#[test]
fn negative_offsets_from_maximum_origins_are_rejected_before_access() {
    check_case("row_negative_invalid", (i32::MAX - 1, 0), (i32::MIN, 0));
    check_case("col_negative_invalid", (0, i32::MAX - 1), (0, i32::MIN));
}
#[test]
fn large_positive_offsets_from_minimum_origins_are_rejected_before_access() {
    let lower = (i32::MIN, i32::MIN);
    check_case("minimum_row_invalid", lower, (i32::MAX, i32::MIN));
    check_case("minimum_col_invalid", lower, (i32::MIN, i32::MAX));
}

#[test]
fn valid_widened_shape_offsets_still_check_actual_matrix_backing() {
    use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
    let mut array = Array2::<i32>::with_size(3, 3);
    array.update_lower_row(i32::MAX - 1);
    array.update_lower_col(i32::MAX - 1);
    *array.change_array1() = Array1::with_size(1);
    assert_eq!(array.value(i32::MAX - 1, i32::MAX - 1), Ok(&0));
    assert!(matches!(
        array.value(i32::MAX, i32::MAX),
        Err(ArrayError::Index { index: 4, .. })
    ));
    assert!(array.change_value(i32::MAX, i32::MAX).is_err());
}
#[test]
fn rejected_indices_preserve_existing_wrapped_error_bound_payloads() {
    use rustycad_tkernel::ncollection::{Array2, ArrayError};
    let mut array = Array2::<i32>::with_size(3, 3);
    array.update_lower_row(i32::MAX - 1);
    assert_eq!(
        array.value(i32::MIN, 0),
        Err(ArrayError::Index {
            index: i32::MIN.into(),
            lower: (i32::MAX - 1).into(),
            upper: i32::MIN.into(),
        })
    );
}
#[test]
fn empty_huge_row_shape_rejects_column_without_evaluating_overflowing_upper_row() {
    use rustycad_tkernel::ncollection::{Array2, ArrayError};
    let mut array = Array2::<i32>::with_size(usize::MAX, 0);
    array.update_lower_row(-2);
    assert_eq!(
        array.value(0, 0),
        Err(ArrayError::Index {
            index: 0,
            lower: 0,
            upper: -1
        })
    );
    assert!(array.as_slice().is_empty());
    let actual = format!("{};{}", u8::from(array.value(0, 0).is_err()), array.len());
    support::native_row("empty_huge_shape", &actual);
}
