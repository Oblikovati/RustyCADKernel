//! Type policy and safe error/empty domains of Array2's mutable donor resizer.
use rustycad_tkernel::ncollection::{Array2, ArrayError};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Default)]
struct MoveOnly(i32);

fn move_value(target: &mut MoveOnly, donor: &mut MoveOnly) {
    target.0 = donor.0;
    donor.0 = -donor.0;
}

#[test]
fn explicit_resize_assignment_does_not_require_clone() {
    let mut buffer = [1, 2, 3, 4, 5, 6].map(MoveOnly);
    let mut array = Array2::from_buffer(&mut buffer, 2, 3).unwrap();
    array
        .resize_trim_with_assignment(2, 2, true, move_value)
        .unwrap();
    assert_eq!(
        array
            .as_slice()
            .iter()
            .map(|item| item.0)
            .collect::<Vec<_>>(),
        [1, 2, 4, 5]
    );
    drop(array);
    assert_eq!(buffer.map(|item| item.0), [-1, -2, 3, -4, -5, 6]);
}

#[test]
fn no_copy_resize_defaults_without_clone_or_assignment() {
    let mut array = Array2::<MoveOnly>::with_size(1, 2);
    array
        .resize_with_assignment(2, 2, false, |_, _| {
            panic!("expected no assignment for copy=false")
        })
        .unwrap();
    assert_eq!(array.len(), 4);
    assert!(array.as_slice().iter().all(|item| item.0 == 0));
}

#[test]
fn invalid_bounds_leave_the_borrowed_rectangle_unchanged() {
    let mut buffer = [1, 2, 3, 4];
    let mut array = Array2::from_buffer_bounds(&mut buffer, 3, 4, 5, 6).unwrap();
    let pointer = array.data_ptr();
    let result = array.resize_bounds_with_assignment(3, 2, 5, 6, true, |_, _| {
        panic!("expected bounds validation first")
    });
    assert!(matches!(
        result,
        Err(ArrayError::Bounds { lower: 3, upper: 2 })
    ));
    assert_eq!((array.lower_row(), array.lower_col()), (3, 5));
    assert_eq!(array.data_ptr(), pointer);
    assert_eq!(array.as_slice(), [1, 2, 3, 4]);
}

struct DefaultFailure(i32);
impl Default for DefaultFailure {
    fn default() -> Self {
        panic!("default construction requested: expected the configured first-constructor failure");
    }
}

#[test]
fn default_failure_after_move_out_retains_the_native_empty_receiver_state() {
    let mut buffer = [DefaultFailure(1), DefaultFailure(2)];
    let mut array = Array2::from_buffer(&mut buffer, 1, 2).unwrap();
    let failure = catch_unwind(AssertUnwindSafe(|| {
        array.resize_bounds_with_assignment(3, 4, 5, 6, true, |_, _| {
            panic!("expected construction failure before assignment")
        })
    }));
    let state = default_failure_state(&array, failure.is_err());
    drop(array);
    let row = format!("default_failure={state};{},{}", buffer[0].0, buffer[1].0);
    let expected = include_str!("../../../tools/oracle/tkernel_array2_resize_assignment.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native row: {row}"
    );
}

#[test]
fn zero_trim_count_keeps_flat_origin_and_skips_callbacks() {
    let mut buffer = [1, 2, 3, 4];
    let mut array = Array2::from_buffer_bounds(&mut buffer, -2, -1, 3, 4).unwrap();
    array
        .resize_trim_with_assignment(2, 0, true, |_, _| {
            panic!("expected no assignment for empty result")
        })
        .unwrap();
    assert_eq!((array.nb_rows(), array.nb_columns()), (2, 0));
    assert_eq!((array.array1().lower(), array.array1().upper()), (-1, -2));
    assert!(array.data_ptr().is_null());
    drop(array);
    assert_eq!(buffer, [1, 2, 3, 4]);
}

fn default_failure_state(array: &Array2<'_, DefaultFailure>, failed: bool) -> String {
    format!(
        "{},{};{},{};{},{};{};;{}",
        array.nb_rows(),
        array.nb_columns(),
        array.lower_row(),
        array.lower_col(),
        array.array1().lower(),
        array.array1().upper(),
        u8::from(array.is_deletable()),
        u8::from(failed)
    )
}

#[test]
fn resize_policy_keeps_array_send_sync_unpin_and_storage_lifetime_covariance() {
    fn assert_traits<T: Send + Sync + Unpin>() {}
    fn shorten<'a>(array: Array2<'static, i32>) -> Array2<'a, i32> {
        array
    }
    assert_traits::<Array2<'static, String>>();
    let mut array = shorten(Array2::<i32>::with_size(1, 1));
    array
        .resize_with_assignment(1, 2, true, |target, donor| *target = *donor)
        .unwrap();
    assert_eq!(array.len(), 2);
}
