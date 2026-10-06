//! Safe facade mutations enter the raw engine only with checked layout and exclusive ownership.
use super::lifecycle::sparse_with_destruction;
use super::{
    TypedSparseStorage,
    layout::{sparse_checked_bytes, sparse_construct, sparse_project},
};
use std::{cell::Cell, ffi::c_void};
impl<T> TypedSparseStorage<T> {
    pub(in super::super) fn insert_owned(&mut self, index: usize, value: T) -> &mut T {
        self.validate_index(index);
        let incoming = Cell::new(Some(value));
        let address = unsafe {
            self.base.set_value_with(
                index,
                std::ptr::null_mut(),
                |slot, _| sparse_construct(slot, incoming.take().unwrap()),
                |slot, _| drop(sparse_project::<T>(slot).replace(incoming.take().unwrap())),
            )
        };
        unsafe { &mut *sparse_project::<T>(address) }
    }
    pub(in super::super) fn remove(&mut self, index: usize) -> bool {
        sparse_with_destruction::<T, _>(|destroy| unsafe { self.base.unset_with(index, destroy) })
    }
    pub(in super::super) fn clear(&mut self) {
        sparse_with_destruction::<T, _>(|destroy| unsafe { self.base.clear_with(destroy) });
    }
    pub(in super::super) fn validate_index(&self, index: usize) {
        let increment = self.base.block_size();
        assert!(
            increment != 0,
            "SparseArray block size {increment}; expected positive size for indexed access"
        );
        sparse_checked_bytes(increment, self.base.item_size());
        let mut slots = self.base.block_slots().max(increment);
        while index / increment >= slots {
            slots = slots
                .checked_mul(2)
                .unwrap_or_else(|| panic!("SparseArray index {index}; expected addressable table"));
        }
        sparse_validate_table(slots, increment);
    }
}
fn sparse_validate_table(slots: usize, increment: usize) {
    let bytes = slots.checked_mul(size_of::<*mut c_void>());
    assert!(
        bytes.is_some_and(|n| n <= isize::MAX as usize),
        "SparseArray table {slots} slots; expected addressable pointer layout"
    );
    assert!(
        slots.checked_mul(increment).is_some(),
        "SparseArray table {slots} blocks of {increment}; expected representable capacity"
    );
}
impl<T: Clone> TypedSparseStorage<T> {
    pub(in super::super) fn insert_borrowed(&mut self, index: usize, value: &T) -> &mut T {
        self.validate_index(index);
        let source = std::ptr::from_ref(value).cast_mut().cast();
        let address = unsafe {
            self.base.set_value_with(
                index,
                source,
                |slot, _| sparse_construct(slot, value.clone()),
                |slot, _| (&mut *sparse_project::<T>(slot)).clone_from(value),
            )
        };
        unsafe { &mut *sparse_project::<T>(address) }
    }
    pub(in super::super) fn assign(&mut self, source: &Self) {
        sparse_with_destruction::<T, _>(|destroy| unsafe {
            self.base.assign_with(
                &source.base,
                |slot, from| sparse_construct(slot, (&*sparse_project::<T>(from)).clone()),
                destroy,
                |slot, from| {
                    (&mut *sparse_project::<T>(slot)).clone_from(&*sparse_project::<T>(from))
                },
            )
        });
    }
}
