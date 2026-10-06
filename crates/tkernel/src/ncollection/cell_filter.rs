//! Spatial hash cells with native truncation and newest-first inspection.
use super::{Array1, BaseAllocator, HashPolicy};
mod cell;
mod cell_map;
pub use cell::{CellFilterCell, CellFilterCellHasher};
use cell_map::CellMap;
use std::marker::PhantomData;
mod cell_index;
mod fallible;
mod geometry;
mod indices;
mod protected;
mod setup;
use cell_index::CellIndex;
pub use geometry::CellGeometry;
use indices::CellIndices;
/// Callback decision; returning `CellAction::Purge` destroys this cell's target copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CellAction {
    /// Preserve this target in the cell.
    Keep,
    /// Remove this target from the current cell.
    Purge,
}
/// Geometry and inspection policy; each target may occur in multiple cells.
pub trait CellInspector {
    /// Copyable target payload, e.g. an object index.
    type Target: Clone;
    /// Point representation, e.g. `[f64; 3]`.
    type Point;
    /// Default spatial dimension; `3` for a three-dimensional policy.
    const DIMENSION: usize;
    /// Extract a coordinate; `coord(0, point)` selects the first axis.
    fn coord(axis: usize, point: &Self::Point) -> f64;
    /// Identify all copies for removal; `is_equal(first, second)`.
    fn is_equal(first: &Self::Target, second: &Self::Target) -> bool;
    /// Examine one candidate; return `CellAction::Keep` to retain it.
    fn inspect(&mut self, target: &Self::Target) -> CellAction;
}
#[derive(Clone, Copy)]
struct CellHash;
impl HashPolicy<CellIndex> for CellHash {
    fn hash(&self, key: &CellIndex) -> usize {
        Self::hash_coordinates(key.as_slice())
    }
    fn equal(&self, first: &CellIndex, second: &CellIndex) -> bool {
        first.as_slice() == second.as_slice()
    }
}
impl CellHash {
    fn hash_coordinates(coordinates: &[i32]) -> usize {
        // OCCT 3d097a03 hashes CellIndex storage directly; copying bytes adds an allocation.
        let bytes: &[u8] =
            crate::standard::hashing::object_bytes(coordinates, std::mem::size_of_val(coordinates));
        crate::standard::hashing::hash_bytes(bytes)
    }
}
/// Hash-indexed cells owning independent target copies; `CellFilter::<Inspector>::new(1.0)`.
///
/// As natively, the range methods iterate the first `dimension` axes (fixed at
/// construction), while a cell index has one entry per cell size, whose count
/// `reset_sizes` may change.
pub struct CellFilter<I: CellGeometry> {
    dimension: usize,
    sizes: Array1<'static, f64>,
    cells: CellMap<I::CellTarget>,
    resource: BaseAllocator,
    inspector: PhantomData<I>,
}
impl<I: CellGeometry> CellFilter<I> {
    /// Add a copy to one cell; `filter.add(&target, &point)`.
    pub fn add(&mut self, target: &I::CellTarget, point: &I::CellPoint) {
        self.add_cell(self.indices(point), target);
    }
    /// Add an independent copy to every covered cell; `filter.add_range(&target, &min, &max)`.
    pub fn add_range(&mut self, target: &I::CellTarget, min: &I::CellPoint, max: &I::CellPoint) {
        for index in self.range(min, max) {
            self.add_cell(index, target);
        }
    }
    /// Remove all equal targets in one cell; `filter.remove(&target, &point)`.
    pub fn remove(&mut self, target: &I::CellTarget, point: &I::CellPoint) {
        self.retain_cell(&self.indices(point), |stored| {
            !I::targets_equal(stored, target)
        });
    }
    /// Remove equal targets throughout an inclusive range; `filter.remove_range(&target, &min, &max)`.
    pub fn remove_range(&mut self, target: &I::CellTarget, min: &I::CellPoint, max: &I::CellPoint) {
        for index in self.range(min, max) {
            self.retain_cell(&index, |stored| !I::targets_equal(stored, target));
        }
    }
    fn indices(&self, point: &I::CellPoint) -> CellIndex {
        CellIndex::from_point::<I>(point, &self.sizes)
    }
    fn range(&self, min: &I::CellPoint, max: &I::CellPoint) -> CellIndices {
        // Native iterateAdd/Remove/Inspect start at axis myDim - 1; axes beyond the cell
        // index length would read past the native LocalArray, so they are clamped.
        let axes: usize = self.dimension.min(self.sizes.len());
        CellIndices::new(self.indices(min), self.indices(max), axes)
    }
    fn add_cell(&mut self, index: CellIndex, target: &I::CellTarget) {
        self.cells.add(&index, target);
    }
    fn retain_cell(&mut self, index: &CellIndex, keep: impl FnMut(&I::CellTarget) -> bool) {
        self.retain_index(index.as_slice(), keep);
    }
}

impl<I: CellInspector> CellFilter<I> {
    /// Inspect newest-first, applying purge decisions immediately; `filter.inspect(&point, &mut inspector)`.
    pub fn inspect(&mut self, point: &I::Point, inspector: &mut I) {
        self.retain_cell(&self.indices(point), |stored| {
            inspector.inspect(stored) == CellAction::Keep
        });
    }
    /// Visit cells with axis zero varying fastest; `filter.inspect_range(&min, &max, &mut inspector)`.
    pub fn inspect_range(&mut self, min: &I::Point, max: &I::Point, inspector: &mut I) {
        for index in self.range(min, max) {
            self.retain_cell(&index, |stored| {
                inspector.inspect(stored) == CellAction::Keep
            });
        }
    }
}

#[cfg(test)]
mod method_tests;

#[cfg(test)]
mod size_storage_tests;

#[cfg(test)]
mod index_storage_tests;

#[cfg(test)]
mod range_storage_tests;

#[cfg(test)]
mod hash_storage_tests;
