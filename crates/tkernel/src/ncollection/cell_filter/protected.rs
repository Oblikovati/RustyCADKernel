//! Callable adapters for native protected helpers, borrowing the caller's index storage.
use super::{CellAction, CellFilter, CellGeometry, CellInspector};
use crate::standard::Failure;
mod walk;
use walk::walk_cell_indices;

impl<I: CellGeometry> CellFilter<I> {
    /// Native protected `add(index, target)`; `filter.add_index(&[2, 3], &target)` skips point conversion.
    pub fn add_index(&mut self, index: &[i32], target: &I::CellTarget) {
        self.cells.add_index(index, target);
    }
    /// Native protected `remove(cell, target)`; `filter.remove_index(&[2, 3], &target)` matches all copies.
    pub fn remove_index(&mut self, index: &[i32], target: &I::CellTarget) {
        self.retain_index(index, |stored| !I::targets_equal(stored, target));
    }
    /// Native protected recursion through `axis`, leaving later cursor coordinates unchanged.
    /// `filter.iterate_add_indices(0, &mut cursor, &min, &max, &target)?` changes only axis zero.
    /// Out-of-bounds accesses return an error before mutation; native access is undefined.
    /// A reversed outer range returns without accessing the cursor, as natively.
    pub fn iterate_add_indices(
        &mut self,
        axis: i32,
        cursor: &mut [i32],
        min: &[i32],
        max: &[i32],
        target: &I::CellTarget,
    ) -> Result<(), Failure> {
        walk_cell_indices(axis, cursor, min, max, |index| {
            self.add_index(index, target)
        })
    }
    /// Native protected removal recursion; `filter.iterate_remove_indices(1, &mut cursor, &min, &max, &target)?`.
    /// The cursor retains native partial writes if a target callback panics.
    pub fn iterate_remove_indices(
        &mut self,
        axis: i32,
        cursor: &mut [i32],
        min: &[i32],
        max: &[i32],
        target: &I::CellTarget,
    ) -> Result<(), Failure> {
        walk_cell_indices(axis, cursor, min, max, |index| {
            self.remove_index(index, target)
        })
    }
    pub(super) fn retain_index(
        &mut self,
        index: &[i32],
        mut keep: impl FnMut(&I::CellTarget) -> bool,
    ) {
        let result = self.cells.try_retain_index(index, |stored| {
            Ok::<bool, std::convert::Infallible>(keep(stored))
        });
        match result {
            Ok(()) => (),
            Err(never) => match never {},
        }
    }
}
impl<I: CellInspector> CellFilter<I> {
    /// Native protected `inspect(cell, inspector)`; `filter.inspect_index(&[2, 3], &mut inspector)`.
    pub fn inspect_index(&mut self, index: &[i32], inspector: &mut I) {
        self.retain_index(index, |stored| {
            inspector.inspect(stored) == CellAction::Keep
        });
    }
    /// Native protected inspection recursion; `filter.iterate_inspect_indices(1, &mut cursor, &min, &max, &mut inspector)?`.
    /// Axis zero varies fastest; the cursor remains at the failing cell if inspection panics.
    pub fn iterate_inspect_indices(
        &mut self,
        axis: i32,
        cursor: &mut [i32],
        min: &[i32],
        max: &[i32],
        inspector: &mut I,
    ) -> Result<(), Failure> {
        walk_cell_indices(axis, cursor, min, max, |index| {
            self.inspect_index(index, inspector)
        })
    }
}
