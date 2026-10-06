//! Native CellIndex uses ten inline integers; lookup copies must not alias stored keys.
use super::{CellGeometry, indices::cell_coordinate};
use crate::ncollection::{Array1, LocalArray};
use std::fmt;

pub(super) struct CellIndex(LocalArray<i32, 10>);
impl CellIndex {
    pub(super) fn from_point<I: CellGeometry>(
        point: &I::CellPoint,
        sizes: &Array1<'_, f64>,
    ) -> Self {
        let mut index = Self(LocalArray::new(sizes.len()));
        for (axis, size) in sizes.as_slice().iter().enumerate() {
            index.0[axis] = cell_coordinate(I::cell_coordinate(axis, point) / size);
        }
        index
    }
    pub(super) fn from_slice(source: &[i32]) -> Self {
        let mut index = Self(LocalArray::new(source.len()));
        index.0.as_mut_slice().copy_from_slice(source);
        index
    }
    pub(super) fn as_slice(&self) -> &[i32] {
        self.0.as_slice()
    }
}
impl fmt::Debug for CellIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(formatter)
    }
}
