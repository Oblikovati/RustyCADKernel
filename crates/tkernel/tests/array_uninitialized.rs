#![allow(unsafe_code)]
//! Only initialized writes are observed; native uninitialized reads remain undefined.
#[path = "array_uninitialized/support.rs"]
mod support;
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::mem::MaybeUninit;
use support::{UninitCell, assert_native_row, initialized_linear, initialized_matrix};

#[test]
fn linear_uninitialized_size_constructors_match_native_after_defined_writes() {
    for size in [0, 1, 4] {
        let slots = Array1::<UninitCell>::with_uninit_size(size);
        assert_native_row(&format!("linear_size_{size}"), &initialized_linear(slots));
    }
}
#[test]
fn linear_uninitialized_bounds_preserve_negative_and_empty_ranges() {
    for (lower, upper) in [(-2, 1), (3, 2)] {
        let slots = Array1::<UninitCell>::with_uninit_bounds(lower, upper).unwrap();
        assert_native_row(
            &format!("linear_bounds_{lower}_{upper}"),
            &initialized_linear(slots),
        );
    }
}
#[test]
fn matrix_uninitialized_dimensions_preserve_zero_rows_or_columns() {
    for (rows, cols) in [(0, 3), (2, 0), (2, 3)] {
        let slots = Array2::<UninitCell>::with_uninit_size(rows, cols);
        assert_native_row(
            &format!("matrix_size_{rows}_{cols}"),
            &initialized_matrix(slots),
        );
    }
}
#[test]
fn matrix_uninitialized_bounds_preserve_the_native_flat_origin() {
    for (rl, ru, cl, cu) in [(-2, -1, 3, 4), (-2, -3, 3, 4)] {
        let slots = Array2::<UninitCell>::with_uninit_bounds(rl, ru, cl, cu).unwrap();
        assert_native_row(
            &format!("matrix_bounds_{rl}_{ru}_{cl}_{cu}"),
            &initialized_matrix(slots),
        );
    }
}
#[test]
fn borrowed_linear_transition_preserves_buffer_and_ignores_unborrowed_tail() {
    let mut buffer = [MaybeUninit::<UninitCell>::uninit(); 5];
    let slots = Array1::from_slice_bounds(&mut buffer, -2, 1).unwrap();
    assert_native_row("borrowed_linear", &initialized_linear(slots));
    // SAFETY: the borrowed prefix's first slot was initialized; the tail is not read.
    assert_eq!(unsafe { buffer[0].assume_init() }, UninitCell(11));
}
#[test]
fn borrowed_matrix_transition_preserves_the_exclusive_buffer_lifetime() {
    let mut buffer = [MaybeUninit::<UninitCell>::uninit(); 5];
    let slots = Array2::from_buffer_bounds(&mut buffer, -2, -1, 3, 4).unwrap();
    assert_native_row("borrowed_matrix", &initialized_matrix(slots));
    // SAFETY: only the initialized borrowed prefix is inspected after the view is dropped.
    assert_eq!(unsafe { buffer[3].assume_init() }, UninitCell(32));
}
#[test]
fn matrix_transition_requires_and_preserves_the_entire_changed_backing() {
    for length in [0, 2, 6] {
        let mut slots = Array2::<UninitCell>::with_uninit_size(2, 2);
        *slots.change_array1() = Array1::<UninitCell>::with_uninit_size(length);
        assert_native_row(
            &format!("changed_base_{length}"),
            &initialized_matrix(slots),
        );
    }
}
#[test]
fn invalid_uninitialized_ranges_reject_before_allocation() {
    let first = Array1::<UninitCell>::with_uninit_bounds(3, 1);
    let second = Array2::<UninitCell>::with_uninit_bounds(0, 1, 4, 2);
    assert!(matches!(
        first,
        Err(ArrayError::Bounds { lower: 3, upper: 1 })
    ));
    assert!(matches!(
        second,
        Err(ArrayError::Bounds { lower: 4, upper: 2 })
    ));
}
#[test]
#[should_panic(expected = "expected a product fitting usize")]
fn overflowing_uninitialized_rectangle_has_a_shape_diagnostic() {
    let rows = usize::MAX;
    let _slots = Array2::<UninitCell>::with_uninit_size(rows, 2);
}
#[test]
fn initialization_transition_preserves_references_with_nonstatic_lifetimes() {
    let text = String::from("local");
    let mut slots = Array1::<&str>::with_uninit_size(1);
    slots.as_mut_slice()[0].write(text.as_str());
    // SAFETY: the only slot contains a live reference whose lifetime remains in the type.
    let values = unsafe { slots.assume_init() };
    assert_eq!(values.as_slice(), [text.as_str()]);
}
#[test]
fn shared_transition_retains_read_only_storage_until_explicit_mutation() {
    let buffer = [
        MaybeUninit::new(UninitCell(7)),
        MaybeUninit::new(UninitCell(9)),
    ];
    let slots = Array1::from_shared(&buffer, -3);
    let pointer = slots.data_ptr().cast::<UninitCell>();
    // SAFETY: both source slots were initialized and remain immutable for the view lifetime.
    let mut values = unsafe { slots.assume_init() };
    assert_eq!(values.data_ptr(), pointer);
    assert!(!values.is_deletable());
    values.as_mut_slice()[0] = UninitCell(12);
    assert!(values.is_deletable());
    // SAFETY: the original shared buffer remains initialized and unchanged after detachment.
    assert_eq!(unsafe { buffer[0].assume_init() }, UninitCell(7));
}
#[test]
fn zero_sized_slots_transfer_without_pointer_or_length_changes() {
    let mut slots = Array1::<()>::with_uninit_size(4);
    for slot in slots.as_mut_slice() {
        slot.write(());
    }
    let pointer = slots.data_ptr().cast::<()>();
    // SAFETY: all four logical ZST slots were explicitly initialized.
    let values = unsafe { slots.assume_init() };
    assert_eq!(values.len(), 4);
    assert_eq!(values.data_ptr(), pointer);
}
