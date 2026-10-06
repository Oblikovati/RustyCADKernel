//! Native CellIndex uses ten inline integers; lookup copies must not alias stored keys.
#![allow(unsafe_code)]
use super::{CellGeometry, indices::cell_coordinate};
use crate::ncollection::{Array1, BaseAllocator};
use std::{fmt, ptr::NonNull};
mod storage;
/// Native field order: ten inline integers, pointer slot, logical size.
/// The pointer slot is null for inline storage: Rust moves must not retain a self pointer.
#[repr(C)]
pub(super) struct CellIndex {
    inline: [i32; 10],
    heap: Option<NonNull<i32>>,
    length: usize,
}
impl CellIndex {
    pub(super) fn from_point<I: CellGeometry>(
        point: &I::CellPoint,
        sizes: &Array1<'_, f64>,
    ) -> Self {
        let mut index = Self::allocate(sizes.len());
        for (axis, size) in sizes.as_slice().iter().enumerate() {
            index.as_mut_slice()[axis] = cell_coordinate(I::cell_coordinate(axis, point) / size);
        }
        index
    }
    pub(super) fn from_slice(source: &[i32]) -> Self {
        let mut index = Self::allocate(source.len());
        index.as_mut_slice().copy_from_slice(source);
        index
    }
    pub(super) fn as_mut_slice(&mut self) -> &mut [i32] {
        let pointer = self
            .heap
            .map_or_else(|| self.inline.as_mut_ptr(), NonNull::as_ptr);
        // All coordinates are initialized, including while a point callback unwinds.
        unsafe { std::slice::from_raw_parts_mut(pointer, self.length) }
    }
    pub(super) fn as_slice(&self) -> &[i32] {
        unsafe { std::slice::from_raw_parts(self.data_ptr(), self.length) }
    }
    pub(super) fn data_ptr(&self) -> *const i32 {
        self.heap
            .map_or_else(|| self.inline.as_ptr(), |heap| heap.as_ptr().cast_const())
    }
}
impl fmt::Debug for CellIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(formatter)
    }
}
impl Clone for CellIndex {
    fn clone(&self) -> Self {
        // Native Cell(CellIndex) copies coordinates into its own ten-inline-element storage.
        Self::from_slice(self.as_slice())
    }
}
impl Drop for CellIndex {
    fn drop(&mut self) {
        self.release_heap();
    }
}
// SAFETY: the optional pointer uniquely owns initialized i32 storage; shared access is readonly
// and mutation requires &mut CellIndex. No thread-affine allocator state is stored here.
unsafe impl Send for CellIndex {}
unsafe impl Sync for CellIndex {}
#[cfg(test)]
mod native_tests;
