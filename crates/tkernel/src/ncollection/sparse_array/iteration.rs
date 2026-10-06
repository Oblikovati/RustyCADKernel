//! Safe borrowing iterators traverse each occupied raw slot exactly once.
use super::{
    SparseArray,
    storage::{TypedSparseStorage, first_raw_sparse_key, sparse_project},
};
use crate::ncollection::SparseArrayBase;
use std::marker::PhantomData;
pub(super) struct SparseEntries<'a, T> {
    storage: &'a TypedSparseStorage<T>,
    next: usize,
}
impl<'a, T> SparseEntries<'a, T> {
    pub(super) fn new(array: &'a SparseArray<T>) -> Self {
        Self {
            storage: &array.storage,
            next: 0,
        }
    }
}
impl<'a, T> Iterator for SparseEntries<'a, T> {
    type Item = (usize, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.storage.first_key(self.next)?;
        self.next = key + 1;
        Some((key, self.storage.value(key).unwrap()))
    }
}
pub(super) struct SparseEntriesMut<'a, T> {
    base: &'a SparseArrayBase,
    next: usize,
    exclusive: PhantomData<&'a mut T>,
}
impl<'a, T> SparseEntriesMut<'a, T> {
    pub(super) fn new(array: &'a mut SparseArray<T>) -> Self {
        Self {
            base: &array.storage.base,
            next: 0,
            exclusive: PhantomData,
        }
    }
}
// SAFETY: the exclusive array borrow freezes all blocks and metadata; only next() yields each
// owned T once. Sending iteration transfers access to those T, while shared access never edits.
unsafe impl<T: Send> Send for SparseEntriesMut<'_, T> {}
unsafe impl<T: Sync> Sync for SparseEntriesMut<'_, T> {}
// Borrowing the iterator itself across unwind does not permit mutation; preserve std IterMut.
impl<T: std::panic::RefUnwindSafe> std::panic::RefUnwindSafe for SparseEntriesMut<'_, T> {}
impl<T> Unpin for SparseEntriesMut<'_, T> {}
impl<'a, T> Iterator for SparseEntriesMut<'a, T> {
    type Item = (usize, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        let key = first_raw_sparse_key(self.base, self.next)?;
        self.next = key + 1;
        let address = self.base.get_value(key).ok()?;
        // SAFETY: the exclusive owner borrow outlives every result; keys never repeat.
        Some((key, unsafe { &mut *sparse_project::<T>(address) }))
    }
}
