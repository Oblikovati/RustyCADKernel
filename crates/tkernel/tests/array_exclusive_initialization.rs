#![allow(unsafe_code)]
//! Exclusive initialization is a Rust ownership transition, not a new native method.
#[path = "array_exclusive_initialization/support.rs"]
mod support;
use rustycad_tkernel::ncollection::{Array1, Array2};
use std::cell::Cell;
use std::mem::MaybeUninit;
use support::{
    DropCell, changed_matrix_case, check_linear, copy_source_slots, destroy_slots,
    initialize_slots, rectangle,
};

#[test]
fn owned_raw_constructed_noncopy_values_transfer_once_without_element_operations() {
    let (deaths, copies) = (Cell::new(0), Cell::new(0));
    let source: [_; 3] = std::array::from_fn(|value| DropCell {
        value,
        deaths: &deaths,
    });
    let slots = copy_source_slots(&source, &copies);
    let pointer = slots.data_ptr();
    let values = unsafe { slots.try_assume_init_exclusive() }.unwrap_or_else(|_| panic!("shared"));
    check_linear(&values, pointer.cast(), true);
    assert_eq!((copies.get(), deaths.get()), (3, 0));
    drop(values);
    assert_eq!(deaths.get(), 3);
    drop(source);
    assert_eq!(deaths.get(), 6);
}

#[test]
fn borrowed_noncopy_values_keep_lifetime_and_external_drop_responsibility() {
    let deaths = Cell::new(0);
    let mut buffer: [_; 4] = std::array::from_fn(|_| MaybeUninit::uninit());
    initialize_slots(&mut buffer[..3], &deaths);
    let slots = Array1::from_slice_bounds(&mut buffer, -2, 0).unwrap();
    let pointer = slots.data_ptr().cast();
    let values = unsafe { slots.try_assume_init_exclusive() }.unwrap_or_else(|_| panic!("shared"));
    check_linear(&values, pointer, false);
    drop(values);
    assert_eq!(deaths.get(), 0);
    unsafe { destroy_slots(&mut buffer[..3]) };
    assert_eq!(deaths.get(), 3);
}

#[test]
fn matrix_transition_preserves_shortened_extended_and_empty_actual_backing() {
    for length in [0, 2, 6] {
        changed_matrix_case(length);
    }
}

#[test]
fn borrowed_matrix_does_not_drop_external_values_or_expose_uninitialized_tail() {
    let deaths = Cell::new(0);
    let mut buffer: [_; 5] = std::array::from_fn(|_| MaybeUninit::uninit());
    initialize_slots(&mut buffer[..4], &deaths);
    let slots = Array2::from_buffer_bounds(&mut buffer, -2, -1, 3, 4).unwrap();
    let (pointer, bounds) = (slots.data_ptr().cast::<DropCell<'_>>(), rectangle(&slots));
    let values = unsafe { slots.try_assume_init_exclusive() }.unwrap_or_else(|_| panic!("shared"));
    assert_eq!((values.data_ptr(), rectangle(&values)), (pointer, bounds));
    assert_eq!(values.len(), 4);
    drop(values);
    assert_eq!(deaths.get(), 0);
    unsafe { destroy_slots(&mut buffer[..4]) };
    assert_eq!(deaths.get(), 4);
}

#[test]
fn rejected_shared_linear_storage_can_still_use_its_original_copy_detach_policy() {
    let buffer = [MaybeUninit::new(7_u32), MaybeUninit::new(9)];
    let slots = Array1::from_shared(&buffer, -3);
    let pointer = slots.data_ptr();
    let returned = unsafe { slots.try_assume_init_exclusive() }.err().unwrap();
    assert_eq!((returned.data_ptr(), returned.lower()), (pointer, -3));
    assert!(!returned.is_deletable());
    let mut values = unsafe { returned.assume_init() };
    values.as_mut_slice()[0] = 12;
    assert_ne!(values.data_ptr(), pointer.cast());
    assert_eq!(unsafe { buffer[0].assume_init() }, 7);
}

#[test]
fn rejected_shared_matrix_retains_independent_shape_and_backing_metadata() {
    let buffer = [MaybeUninit::<u32>::uninit(); 3];
    let mut slots = Array2::<u32>::with_uninit_bounds(-2, -1, 3, 4).unwrap();
    *slots.change_array1() = Array1::from_shared(&buffer, 17);
    let (pointer, bounds) = (slots.data_ptr(), rectangle(&slots));
    let returned = unsafe { slots.try_assume_init_exclusive() }.err().unwrap();
    assert_eq!(
        (returned.data_ptr(), rectangle(&returned)),
        (pointer, bounds)
    );
    assert_eq!(returned.len(), 3);
    assert!(!returned.is_deletable());
}

#[test]
fn rejected_shared_linear_storage_is_not_read_even_when_uninitialized_or_empty() {
    let buffer = [MaybeUninit::<u32>::uninit(); 3];
    for length in [0, 3] {
        let slots = Array1::from_shared(&buffer[..length], -7);
        let pointer = slots.data_ptr();
        let returned = unsafe { slots.try_assume_init_exclusive() }.err().unwrap();
        assert_eq!(
            (returned.data_ptr(), returned.len(), returned.lower()),
            (pointer, length, -7)
        );
    }
}

#[test]
fn empty_owned_and_borrowed_noncopy_storage_retain_pointer_and_bounds() {
    let mut buffer: [MaybeUninit<String>; 0] = [];
    let owned = Array1::<String>::with_uninit_size(0);
    let borrowed = Array1::from_slice_bounds(&mut buffer, -2, -3).unwrap();
    for slots in [owned, borrowed] {
        let (pointer, lower) = (slots.data_ptr().cast::<String>(), slots.lower());
        let values =
            unsafe { slots.try_assume_init_exclusive() }.unwrap_or_else(|_| panic!("shared"));
        assert_eq!((values.data_ptr(), values.lower()), (pointer, lower));
        assert!(values.is_empty());
    }
}

#[test]
fn zero_sized_noncopy_slots_preserve_pointer_and_exact_logical_length() {
    struct Zero;
    let mut slots = Array2::<Zero>::with_uninit_size(2, 3);
    for slot in slots.as_mut_slice() {
        slot.write(Zero);
    }
    let pointer = slots.data_ptr().cast::<Zero>();
    let values = unsafe { slots.try_assume_init_exclusive() }.unwrap_or_else(|_| panic!("shared"));
    assert_eq!((values.data_ptr(), values.len()), (pointer, 6));
    assert_eq!(rectangle(&values), (0, 0, 2, 3, 0));
}
